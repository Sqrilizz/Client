use crate::codec::{DecodeResult, MAX_STRING_LENGTH, Reader};
use crate::item::{ItemCodec, ItemStack, PARTICLE, RESOLVABLE_PROFILE, Shape};

const EOF_MARKER: u8 = 0xFF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Cat,
    Cow,
    Wolf,
    Chicken,
}

#[derive(Debug)]
pub enum Serializer {
    Byte,
    Int,
    Long,
    Float,
    Bool,
    Component,
    OptionalComponent,
    ItemStack,
    OptionalBlockPos,
    VillagerData,
    Pose,
    Variant(Variant),
    Vector3,
    Quaternion,
    Skip(Shape),
}

impl Serializer {
    pub fn skip_native_value(&'static self, r: &mut Reader) -> DecodeResult<bool> {
        let shape: &'static Shape = match self {
            Self::Byte | Self::Bool => {
                r.bytes(1)?;
                return Ok(true);
            }
            Self::Long => {
                r.varlong()?;
                return Ok(true);
            }
            Self::Int | Self::Pose | Self::Variant(_) => &Shape::VarInt,
            Self::Float => &Shape::Float,
            Self::Component => &TEXT,
            Self::OptionalComponent => &Shape::Optional(&TEXT),
            Self::OptionalBlockPos => &Shape::Optional(&Shape::Long),
            Self::VillagerData => &Shape::Struct(&[Shape::VarInt, Shape::VarInt, Shape::VarInt]),
            Self::ItemStack => return Ok(false),
            Self::Vector3 => &Shape::Struct(&[Float, Float, Float]),
            Self::Quaternion => &Shape::Struct(&[Float, Float, Float, Float]),
            Self::Skip(Shape::Ref(shape) | Shape::List(_, shape))
                if std::ptr::eq(*shape, &PARTICLE) =>
            {
                return Ok(false);
            }
            Self::Skip(shape) => shape,
        };
        ItemCodec::native().skip(r, shape)?;
        Ok(true)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Byte(u8),
    Int(i32),
    Long(i64),
    Float(f32),
    Bool(bool),
    Component(Box<[u8]>),
    OptionalComponent(Option<Box<[u8]>>),
    Item(Option<ItemStack>),
    OptionalBlockPos(Option<(i32, i32, i32)>),
    VillagerData {
        kind: u32,
        profession: u32,
        level: u32,
    },
    Pose(u32),
    Variant(Variant, u32),
    Vector3([f32; 3]),
    Quaternion([f32; 4]),
    Other,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataValue {
    pub index: u8,
    pub value: Value,
}

use Serializer as S;
use Shape::*;

const TEXT: Shape = Nbt;
const REGISTRY_ID: Shape = VarInt;
const ENUM: Shape = VarInt;

macro_rules! serializers_26_2 {
    ($($extra:expr),*) => { &[
    S::Byte,
    S::Int,
    S::Long,
    S::Float,
    S::Skip(Utf(MAX_STRING_LENGTH)),
    S::Component,
    S::OptionalComponent,
    S::ItemStack,
    S::Bool,
    S::Skip(Struct(&[Float, Float, Float])),
    S::Skip(Long),
    S::OptionalBlockPos,
    S::Skip(ENUM),
    S::Skip(Optional(&Uuid)),
    S::Skip(VarInt),
    S::Skip(VarInt),
    S::Skip(Ref(&PARTICLE)),
    S::Skip(List(usize::MAX, &PARTICLE)),
    S::VillagerData,
    S::Skip(VarInt),
    S::Pose,
    S::Variant(Variant::Cat),
    S::Skip(REGISTRY_ID),
    S::Variant(Variant::Cow),
    S::Skip(REGISTRY_ID),
    S::Variant(Variant::Wolf),
    S::Skip(REGISTRY_ID),
    S::Skip(REGISTRY_ID),
    S::Skip(REGISTRY_ID),
    S::Skip(REGISTRY_ID),
    S::Variant(Variant::Chicken),
    S::Skip(REGISTRY_ID),
    S::Skip(REGISTRY_ID),
    S::Skip(Optional(&Struct(&[Utf(MAX_STRING_LENGTH), Long]))),
    S::Skip(Holder(&Struct(&[
        VarInt,
        VarInt,
        Utf(MAX_STRING_LENGTH),
        Optional(&TEXT),
        Optional(&TEXT),
    ]))),
    S::Int,
    S::Int,
    S::Int,
    S::Int,
    S::Vector3,
    S::Quaternion,
    S::Skip(Ref(&RESOLVABLE_PROFILE)),
    S::Skip(ENUM),
    $($extra),*
] };
}

pub static SERIALIZERS_26_2: &[Serializer] = serializers_26_2!();
pub static SERIALIZERS_26_3: &[Serializer] = serializers_26_2!(S::Skip(ENUM));

impl ItemCodec {
    pub fn read_entity_data(&self, r: &mut Reader) -> DecodeResult<Vec<DataValue>> {
        let mut values = Vec::new();
        loop {
            let index = r.u8()?;
            if index == EOF_MARKER {
                return Ok(values);
            }
            let serializer = r.varint()?;
            let serializer = self
                .entity_data_serializers()
                .get(serializer as usize)
                .ok_or_else(|| r.error("unknown entity data serializer"))?;
            let value = self.read_data_value(r, serializer)?;
            values.push(DataValue { index, value });
        }
    }

    fn read_data_value(
        &self,
        r: &mut Reader,
        serializer: &'static Serializer,
    ) -> DecodeResult<Value> {
        Ok(match serializer {
            S::Byte => Value::Byte(r.u8()?),
            S::Int => Value::Int(r.var_i32()?),
            S::Long => Value::Long(r.varlong()? as i64),
            S::Float => Value::Float(r.f32()?),
            S::Bool => Value::Bool(r.bool()?),
            S::Component => {
                let start = r.pos();
                self.skip(r, &TEXT)?;
                Value::Component(r.since(start).into())
            }
            S::OptionalComponent => Value::OptionalComponent(r.optional(|r| {
                let start = r.pos();
                self.skip(r, &TEXT)?;
                Ok(r.since(start).into())
            })?),
            S::ItemStack => Value::Item(self.read_stack(r)?),
            S::OptionalBlockPos => Value::OptionalBlockPos(r.optional(Reader::block_pos)?),
            S::VillagerData => Value::VillagerData {
                kind: r.varint()?,
                profession: r.varint()?,
                level: r.varint()?,
            },
            S::Pose => Value::Pose(r.varint()?),
            S::Variant(variant) => Value::Variant(*variant, r.varint()?),
            S::Vector3 => Value::Vector3([r.f32()?, r.f32()?, r.f32()?]),
            S::Quaternion => Value::Quaternion([r.f32()?, r.f32()?, r.f32()?, r.f32()?]),
            S::Skip(shape) => {
                self.skip(r, shape)?;
                Value::Other
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::tests::{codec, id};
    use crate::registries::ClientRegistry;
    use crate::wire::write_varint;

    fn frame(entries: &[(u8, u32, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for (index, serializer, value) in entries {
            out.push(*index);
            write_varint(&mut out, *serializer);
            out.extend_from_slice(value);
        }
        out.push(EOF_MARKER);
        out
    }

    fn read(bytes: &[u8]) -> DecodeResult<Vec<DataValue>> {
        let mut r = Reader::new(bytes);
        let values = codec().read_entity_data(&mut r)?;
        r.ensure_eof()?;
        Ok(values)
    }

    fn values(bytes: &[u8]) -> Vec<Value> {
        read(bytes)
            .expect("valid entity data")
            .into_iter()
            .map(|v| v.value)
            .collect()
    }

    #[test]
    fn table_covers_every_serializer() {
        assert_eq!(SERIALIZERS_26_2.len(), 43);
        assert_eq!(SERIALIZERS_26_3.len(), 44);
    }

    #[test]
    fn decodes_the_values_the_client_reads() {
        let packed = (5i64 << 38) | ((-3i64 & 0x3ff_ffff) << 12) | (-7i64 & 0xfff);
        let pos = [&[1u8][..], &packed.to_be_bytes()].concat();
        let v3 = [
            1.0f32.to_be_bytes(),
            2.0f32.to_be_bytes(),
            3.0f32.to_be_bytes(),
        ]
        .concat();
        let q = [
            0.0f32.to_be_bytes(),
            0.0f32.to_be_bytes(),
            0.0f32.to_be_bytes(),
            1.0f32.to_be_bytes(),
        ]
        .concat();
        let bytes = frame(&[
            (0, 0, &[0x82]),
            (1, 1, &[0xAC, 0x02]),
            (2, 3, &1.5f32.to_be_bytes()),
            (3, 8, &[1]),
            (4, 6, &[1, 8, 0, 2, b'h', b'i']),
            (5, 6, &[0]),
            (6, 20, &[5]),
            (7, 11, &pos),
            (8, 18, &[2, 3, 4]),
            (9, 23, &[7]),
            (10, 7, &[0]),
            (11, 5, &[8, 0, 2, b'h', b'i']),
            (12, 39, &v3),
            (13, 40, &q),
        ]);
        assert_eq!(
            values(&bytes),
            [
                Value::Byte(0x82),
                Value::Int(300),
                Value::Float(1.5),
                Value::Bool(true),
                Value::OptionalComponent(Some([8, 0, 2, b'h', b'i'].into())),
                Value::OptionalComponent(None),
                Value::Pose(5),
                Value::OptionalBlockPos(Some((5, -7, -3))),
                Value::VillagerData {
                    kind: 2,
                    profession: 3,
                    level: 4,
                },
                Value::Variant(Variant::Cow, 7),
                Value::Item(None),
                Value::Component([8, 0, 2, b'h', b'i'].into()),
                Value::Vector3([1.0, 2.0, 3.0]),
                Value::Quaternion([0.0, 0.0, 0.0, 1.0]),
            ]
        );
    }

    #[test]
    fn copper_golem_states_match_vanilla_encoder() {
        let payload = crate::vanilla_fixtures::case("set_entity_data/copper_golem");
        let mut r = Reader::new(&payload);
        assert_eq!(r.var_i32().unwrap(), 7);
        let values = codec().read_entity_data(&mut r).expect("valid entity data");
        r.ensure_eof().unwrap();
        assert_eq!(
            values,
            [
                DataValue {
                    index: 16,
                    value: Value::Int(2),
                },
                DataValue {
                    index: 17,
                    value: Value::Int(3),
                },
            ]
        );
    }

    #[test]
    fn sniffer_state_matches_vanilla_encoder() {
        let payload = crate::vanilla_fixtures::case("set_entity_data/sniffer_state");
        let mut r = Reader::new(&payload);
        assert_eq!(r.var_i32().unwrap(), 7);
        let values = codec().read_entity_data(&mut r).expect("valid entity data");
        r.ensure_eof().unwrap();
        assert_eq!(
            values,
            [DataValue {
                index: 18,
                value: Value::Int(5),
            }]
        );
    }

    #[test]
    fn armadillo_state_matches_vanilla_encoder() {
        let payload = crate::vanilla_fixtures::case("set_entity_data/armadillo_state");
        let mut r = Reader::new(&payload);
        assert_eq!(r.var_i32().unwrap(), 7);
        let values = codec().read_entity_data(&mut r).expect("valid entity data");
        r.ensure_eof().unwrap();
        assert_eq!(
            values,
            [DataValue {
                index: 18,
                value: Value::Int(3),
            }]
        );
    }

    #[test]
    fn decodes_an_item_stack() {
        let stone = id(ClientRegistry::Item, "stone");
        let mut value = vec![3];
        write_varint(&mut value, stone);
        value.extend([0, 0]);
        let [Value::Item(Some(stack))] = &values(&frame(&[(8, 7, &value)]))[..] else {
            panic!("expected one stack");
        };
        assert_eq!((stack.item, stack.count), (stone, 3));
    }

    #[test]
    fn skips_values_by_shape() {
        let dust = id(ClientRegistry::ParticleType, "dust");
        let flame = id(ClientRegistry::ParticleType, "flame");
        let block_source = id(ClientRegistry::PositionSourceType, "block");
        let vibration = id(ClientRegistry::ParticleType, "vibration");
        let mut particles = vec![3];
        write_varint(&mut particles, dust);
        particles.extend([0, 0, 0, 1, 0x3F, 0x80, 0, 0]);
        write_varint(&mut particles, flame);
        write_varint(&mut particles, vibration);
        write_varint(&mut particles, block_source);
        particles.extend([0; 8]);
        particles.push(20);
        let painting = [0, 2, 1, 1, b'a', 1, 8, 0, 1, b't', 0];
        let bytes = frame(&[
            (0, 15, &[0]),
            (1, 19, &[0]),
            (2, 17, &particles),
            (3, 34, &painting),
            (4, 41, &[0; 8]),
            (5, 1, &[9]),
        ]);
        let decoded = values(&bytes);
        assert_eq!(decoded[..5], [const { Value::Other }; 5]);
        assert_eq!(decoded[5], Value::Int(9));
    }

    #[test]
    fn rejects_unknown_serializers_and_truncation() {
        let err = |bytes: &[u8]| read(bytes).map_err(|e| e.what);
        assert_eq!(
            err(&frame(&[(0, 43, &[])])),
            Err("unknown entity data serializer")
        );
        assert_eq!(err(&[0, 3, 0x3F]), Err("truncated"));
        assert_eq!(err(&[0, 1, 1]), Err("truncated"));
    }
}

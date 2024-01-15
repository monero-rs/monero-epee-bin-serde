use serde::{de::Error as DeError, Deserialize, Deserializer, Serializer};
use serde_bytes::ByteBuf;

pub trait ContainerAble {
    const SIZE: usize;

    /// Returns `Self` from bytes.
    ///
    /// `bytes` is guaranteed to be [`Self::SIZE`] long.
    fn from_bytes(bytes: &[u8]) -> Self;

    fn push_bytes(&self, buf: &mut Vec<u8>);
}

pub fn deserialize<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: ContainerAble,
{
    let bytes: ByteBuf = Deserialize::deserialize(d)?;

    if bytes.len() % T::SIZE != 0 {
        return Err(DeError::invalid_length(
            bytes.len(),
            &"A number divisible by the fields size.",
        ));
    }

    let ret = bytes
        .windows(T::SIZE)
        .step_by(T::SIZE)
        .map(T::from_bytes)
        .collect();
    Ok(ret)
}

pub fn serialize<S, T>(t: &[T], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ContainerAble,
{
    let mut bytes = Vec::with_capacity(t.len() * T::SIZE);
    t.iter().for_each(|tt| tt.push_bytes(&mut bytes));

    s.serialize_bytes(&bytes)
}

impl<const N: usize> ContainerAble for [u8; N] {
    const SIZE: usize = N;

    fn from_bytes(bytes: &[u8]) -> Self {
        bytes.try_into().unwrap()
    }

    fn push_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self)
    }
}

macro_rules! int_container_able {
    ($int:ty ) => {
        impl ContainerAble for $int {
            const SIZE: usize = std::mem::size_of::<$int>();

            fn from_bytes(bytes: &[u8]) -> Self {
                <$int>::from_le_bytes(bytes.try_into().unwrap())
            }

            fn push_bytes(&self, buf: &mut Vec<u8>) {
                buf.extend_from_slice(&self.to_le_bytes())
            }
        }
    };
}

int_container_able!(u16);
int_container_able!(u32);
int_container_able!(u64);
int_container_able!(u128);

int_container_able!(i8);
int_container_able!(i16);
int_container_able!(i32);
int_container_able!(i64);
int_container_able!(i128);

#[cfg(test)]
mod tests {
    use rand::random;
    use serde::{Deserialize, Serialize};

    use crate::{container_as_blob, from_bytes, to_bytes};

    #[test]
    fn ser_deser() {
        macro_rules! create_test_struct {
            ($($typ:ident,)+; $($name: ident: $ty:tt,)+ ) => {
                #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
                struct Test {
                   $(
                    #[serde(with = "container_as_blob")]
                    #[serde(default = "Vec::new")]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    $typ: Vec<$typ>,
                   )+
                    $(
                    #[serde(with = "container_as_blob")]
                    #[serde(default = "Vec::new")]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    $name: Vec<$ty>,
                   )+
                }

                impl Test {
                    fn random() -> Test {
                        Test {
                            $($typ: (0_u8.. random()).map(|_| random()).collect(),)+
                            $($name: (0_u8.. random()).map(|_| random()).collect(),)+
                        }
                    }
                }

            }
        }

        create_test_struct!(
            u16,
            u32,
            u64,
            u128,

            i8,
            i16,
            i32,
            i64,
            i128,;

            arr_32: [u8; 32],
            arr_64: [u8; 16],
        );

        let t = Test::random();

        let bytes = to_bytes(&t).unwrap();

        let tt = from_bytes(&bytes).unwrap();

        assert_eq!(t, tt);
    }
}

use anyhow::{Result, anyhow};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

#[derive(Debug, Clone)]
pub enum FieldDesc {
    Scalar(u8),                               // store scalar type code
    ArrayVar(u8),                             // variable-length array of scalar
    ArrayBounded(u8, u32),                    // bounded-length array of scalar
    ArrayFixed(u8, u32),                      // fixed-length array of scalar
    Struct(String, Vec<(String, FieldDesc)>), // ID string, fields
    StructArray(Box<FieldDesc>),              // array of structures
    Union(String, Vec<(String, FieldDesc)>),  // union with ID
    UnionArray(Box<FieldDesc>),
    VariantUnion,
    VariantUnionArray,
    BoundedString(u32),
}

impl FieldDesc {
    pub fn from_bytes(mut input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&mut input);

        let tag = cursor.read_u8()?; // Read the descriptor tag
        let kind = (tag & 0b11110000) >> 4;
        let subtype = (tag & 0b00001100) >> 2;
        let scalar_type = tag & 0b00000011;

        match tag {
            0b10000000 => {
                // Structure
                let id_len = cursor.read_u8()? as usize;
                let mut id_buf = vec![0u8; id_len];
                cursor.read_exact(&mut id_buf)?;
                let id = String::from_utf8(id_buf)?;

                let num_fields = cursor.read_u8()?;
                let mut fields = Vec::with_capacity(num_fields as usize);
                for _ in 0..num_fields {
                    let name_len = cursor.read_u8()? as usize;
                    let mut name_buf = vec![0u8; name_len];
                    cursor.read_exact(&mut name_buf)?;
                    let name = String::from_utf8(name_buf)?;

                    let field_desc_len = cursor.read_u16::<BigEndian>()?;
                    let mut field_buf = vec![0u8; field_desc_len as usize];
                    cursor.read_exact(&mut field_buf)?;
                    let field_desc = FieldDesc::from_bytes(&field_buf)?;
                    fields.push((name, field_desc));
                }

                Ok(FieldDesc::Struct(id, fields))
            }
            0b10000001 => {
                // Union (same logic as struct)
                todo!("Implement union decode")
            }
            0b10000010 => Ok(FieldDesc::VariantUnion),
            0b10001010 => Ok(FieldDesc::VariantUnionArray),
            0b10000110 => {
                let size = cursor.read_u32::<BigEndian>()?;
                Ok(FieldDesc::BoundedString(size))
            }
            _ => {
                // Scalar or arrays
                match subtype {
                    0b00 => Ok(FieldDesc::Scalar(scalar_type)),
                    0b01 => Ok(FieldDesc::ArrayVar(scalar_type)),
                    0b10 => {
                        let bound = cursor.read_u32::<BigEndian>()?;
                        Ok(FieldDesc::ArrayBounded(scalar_type, bound))
                    }
                    0b11 => {
                        let fixed = cursor.read_u32::<BigEndian>()?;
                        Ok(FieldDesc::ArrayFixed(scalar_type, fixed))
                    }
                    _ => Err(anyhow!("Invalid subtype")),
                }
            }
        }
    }
    pub fn into_bytes(&self) -> Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();

        match self {
            FieldDesc::Scalar(t) => {
                let tag = 0b00000000 | (t & 0b11);
                buf.write_u8(tag)?;
            }
            FieldDesc::ArrayVar(t) => {
                let tag = 0b00000100 | (t & 0b11);
                buf.write_u8(tag)?;
            }
            FieldDesc::ArrayBounded(t, size) => {
                let tag = 0b00001000 | (t & 0b11);
                buf.write_u8(tag)?;
                buf.write_u32::<BigEndian>(*size)?;
            }
            FieldDesc::ArrayFixed(t, size) => {
                let tag = 0b00001100 | (t & 0b11);
                buf.write_u8(tag)?;
                buf.write_u32::<BigEndian>(*size)?;
            }
            FieldDesc::VariantUnion => {
                buf.write_u8(0b10000010)?;
            }
            FieldDesc::VariantUnionArray => {
                buf.write_u8(0b10001010)?;
            }
            FieldDesc::BoundedString(size) => {
                buf.write_u8(0b10000110)?;
                buf.write_u32::<BigEndian>(*size)?;
            }
            _ => return Err(anyhow!("Unsupported encode for variant")),
        }

        Ok(buf)
    }
}

#[test]
fn test_parsing() -> Result<()> {
    let desc = FieldDesc::ArrayBounded(0x02, 10);
    let encoded = desc.into_bytes()?;
    let decoded = FieldDesc::from_bytes(&encoded)?;
    println!("Decoded: {:?}", decoded);
    Ok(())
}

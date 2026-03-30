pub mod blob;
pub mod commit;
pub mod store;
pub mod tree;

pub trait GitObject {
    fn object_type(&self) -> &str;
    fn serialize_body(&self) -> Vec<u8>;
}

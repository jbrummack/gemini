use serde::de::Error;

use crate::{
    client::vertex::ModelString,
    google::ai::generativelanguage::v1::{
        Blob, Content, GenerateContentRequest, GenerateContentResponse, GenerationConfig, Part,
        part::Data,
    },
    identical_impl,
};
identical_impl!();

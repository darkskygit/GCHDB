use super::*;
use tantivy::schema::*;

pub struct Fields {
    pub idx: Field,
    pub content: Field,
    pub timestamp: Field,
    #[allow(dead_code)]
    pub custom: Vec<Field>,
    pub schema: Schema,
}

impl Fields {
    fn new<S: ToString>(custom: Vec<(S, TextOptions)>) -> Self {
        let mut schema_builder = Schema::builder();
        let mut custom_field = vec![];
        for (name, options) in custom {
            custom_field.push(schema_builder.add_text_field(name.to_string().as_str(), options));
        }
        Self {
            idx: schema_builder.add_i64_field("idx", FAST | STORED),
            content: schema_builder.add_text_field(
                "content",
                TextOptions::default().set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer(LANG_CN)
                        .set_index_option(IndexRecordOption::WithFreqsAndPositions),
                ),
            ),
            timestamp: schema_builder.add_i64_field("timestamp", FAST),
            custom: custom_field,
            schema: schema_builder.build(),
        }
    }
}

impl Default for Fields {
    fn default() -> Self {
        Self::new::<String>(vec![])
    }
}

pub trait GetDocument {
    fn get_document(&self, fields: &Fields) -> ChatRecordResult<TantivyDocument>;
}

impl GetDocument for Record {
    fn get_document(&self, fields: &Fields) -> ChatRecordResult<TantivyDocument> {
        Ok(doc! {
            fields.idx => self.get_id() as i64,
            fields.content => self.content.as_str(),
            fields.timestamp => self.timestamp
        })
    }
}

use super::*;
use std::time::Instant;
use tantivy::{
    collector::TopDocs, query::QueryParser, schema::Schema, Index, IndexReader, IndexWriter,
    ReloadPolicy,
};

pub struct ContentIndexer {
    fields: Fields,
    index: Index,
    reader: IndexReader,
    writer: IndexWriter,
}

impl ContentIndexer {
    pub fn new() -> ChatRecordResult<Self> {
        let fields = Fields::default();
        let (index, reader) = Self::get_index_handle(fields.schema.clone())?;
        let writer = Self::get_index_writer(&index)?;

        Ok(Self {
            fields,
            index,
            reader,
            writer,
        })
    }

    fn get_index_handle(schema: Schema) -> ChatRecordResult<(Index, IndexReader)> {
        let index = Index::create_in_ram(schema);
        tokenizers_register(index.tokenizers());
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        Ok((index, reader))
    }

    fn get_index_writer(index: &Index) -> ChatRecordResult<IndexWriter> {
        let num = num_cpus::get();
        info!("Indexing thread num: {}", num);
        Ok(index.writer_with_num_threads(num, num * 10_000_000)?)
    }

    pub fn cleanup_index(&mut self) -> ChatRecordResult<()> {
        self.writer.delete_all_documents()?;
        self.writer.commit()?;
        Ok(())
    }

    pub fn gen_index<D: GetDocument>(&mut self, records: Vec<D>) -> ChatRecordResult<()> {
        let total = records.len() as f64;
        let mut last_parent = 0.0;
        let mut sw = Instant::now();
        for (i, metadata) in records.iter().enumerate() {
            self.writer
                .add_document(metadata.get_document(&self.fields)?);
            if total > 200.0 && i as f64 / total - last_parent >= 0.01 {
                last_parent = i as f64 / total;
                info!(
                    "curr parent: {} / {}, {}%, {}ms",
                    i,
                    total,
                    last_parent,
                    sw.elapsed().as_millis()
                );
                sw = Instant::now();
            }
        }
        self.writer.commit()?;
        futures::executor::block_on(self.writer.garbage_collect_files())?;
        self.reader.reload()?;
        Ok(())
    }

    pub fn search(&self, offset: i64, limit: i64, query: &str) -> ChatRecordResult<Vec<i32>> {
        let offset = if offset > 0 { offset as usize } else { 0 };
        let searcher = self.reader.searcher();
        Ok(searcher
            .search(
                &QueryParser::for_index(
                    &self.index,
                    self.fields
                        .custom
                        .iter()
                        .cloned()
                        .chain(vec![self.fields.content])
                        .collect(),
                )
                .parse_query(query)?,
                &TopDocs::with_limit(offset + limit as usize)
                    .order_by_u64_field(self.fields.timestamp),
            )?
            .iter()
            .skip(offset)
            .filter_map(|(_score, doc_address)| {
                use tantivy::schema::Value;
                searcher.doc(*doc_address).ok().and_then(|doc| {
                    doc.get_first(self.fields.idx).and_then(|val| match val {
                        Value::U64(val) => Some(*val as i32),
                        Value::I64(val) => Some(*val as i32),
                        _ => None,
                    })
                })
            })
            .collect())
    }
}

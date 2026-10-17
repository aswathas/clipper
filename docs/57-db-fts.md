# Database: FTS5

SQLite FTS5 is enough for personal-scale search. 10k clips is
under 50 MB of index. Reindex on every write is fine.

Triggers keep the FTS table in step:
```sql
CREATE TRIGGER clips_ai AFTER INSERT ON clips BEGIN
  INSERT INTO clips_fts(rowid, title, body, host, tags)
  VALUES (new.rowid, new.title, new.body, new.host, new.tags);
END;
```

`bm25` is the ranking function. We do not enable the trigram
tokenizer; English + a small ASCII set is the entire corpus.

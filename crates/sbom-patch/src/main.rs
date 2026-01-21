use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        anyhow::bail!("usage: sbom-patch <pack_root> <gtpack_out> <sbom_out>");
    }
    let pack_root = PathBuf::from(&args[1]);
    let gtpack_out = PathBuf::from(&args[2]);
    let sbom_out = PathBuf::from(&args[3]);

    let schema_paths = collect_schema_paths(&pack_root);
    if schema_paths.is_empty() {
        return Ok(());
    }

    let mut sbom = read_sbom_from_zip(&gtpack_out)?;
    ensure_format(&mut sbom);
    patch_sbom_files(&mut sbom, &pack_root, &schema_paths);
    write_sbom_json(&sbom_out, &sbom)?;

    rewrite_gtpack(&gtpack_out, &pack_root, &schema_paths, &sbom)?;

    Ok(())
}

fn collect_schema_paths(pack_root: &Path) -> Vec<String> {
    let schemas_root = pack_root.join("schemas");
    if !schemas_root.exists() {
        return Vec::new();
    }
    let mut paths = Vec::new();
    walk_dir(&schemas_root, &schemas_root, &mut paths);
    paths
}

fn walk_dir(root: &Path, current: &Path, out: &mut Vec<String>) {
    let entries = match fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(root, &path, out);
        } else if let Ok(rel) = path.strip_prefix(root.parent().unwrap_or(root)) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn read_sbom_from_zip(gtpack_out: &Path) -> Result<Value> {
    let file = fs::File::open(gtpack_out).context("open gtpack")?;
    let mut archive = ZipArchive::new(file).context("read gtpack")?;
    let mut sbom_bytes = None;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).context("read zip entry")?;
        if entry.name() == "sbom.cbor" {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).context("read sbom")?;
            sbom_bytes = Some(bytes);
            break;
        }
    }
    let bytes = sbom_bytes.context("sbom.cbor missing")?;
    let sbom: Value = serde_cbor::from_slice(&bytes).context("decode sbom")?;
    Ok(sbom)
}

fn ensure_format(sbom: &mut Value) {
    if sbom.get("format").is_none() {
        sbom.as_object_mut().expect("sbom object").insert(
            "format".to_string(),
            Value::String("greentic-sbom-v1".to_string()),
        );
    }
}

fn patch_sbom_files(sbom: &mut Value, pack_root: &Path, schema_paths: &[String]) {
    let files = sbom
        .get_mut("files")
        .and_then(Value::as_array_mut)
        .expect("sbom files array");

    let existing: HashSet<String> = files
        .iter()
        .filter_map(|entry| entry.get("path").and_then(Value::as_str))
        .map(|path| path.to_string())
        .collect();

    for schema_path in schema_paths {
        if existing.contains(schema_path.as_str()) {
            continue;
        }
        let source = pack_root.join(schema_path);
        let (size, hash_blake3) = hash_blake3(&source);
        files.push(json!({
            "path": schema_path,
            "size": size,
            "hash_blake3": hash_blake3,
            "media_type": "application/json",
        }));
    }
}

fn hash_blake3(path: &Path) -> (u64, String) {
    let bytes = fs::read(path).unwrap_or_default();
    let size = bytes.len() as u64;
    let hash = blake3::hash(&bytes).to_hex().to_string();
    (size, hash)
}

fn write_sbom_json(sbom_out: &Path, sbom: &Value) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(sbom).context("serialize sbom")?;
    fs::write(sbom_out, bytes).context("write sbom json")?;
    Ok(())
}

fn rewrite_gtpack(
    gtpack_out: &Path,
    pack_root: &Path,
    schema_paths: &[String],
    sbom: &Value,
) -> Result<()> {
    let file = fs::File::open(gtpack_out).context("open gtpack")?;
    let mut archive = ZipArchive::new(file).context("read gtpack")?;
    let mut existing = HashSet::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).context("read zip entry")?;
        existing.insert(entry.name().to_string());
    }

    let temp_path = gtpack_out.with_extension("gtpack.tmp");
    let temp_file = fs::File::create(&temp_path).context("create temp gtpack")?;
    let mut writer = ZipWriter::new(temp_file);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).context("read zip entry")?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).context("read entry")?;
        let options = FileOptions::default().compression_method(entry.compression());
        if entry.name() == "sbom.cbor" {
            continue;
        }
        writer
            .start_file(entry.name(), options)
            .context("start entry")?;
        writer.write_all(&bytes).context("write entry")?;
    }

    for schema_path in schema_paths {
        if existing.contains(schema_path) {
            continue;
        }
        let source = pack_root.join(schema_path);
        let bytes = fs::read(&source).context("read schema file")?;
        writer
            .start_file(schema_path, FileOptions::default())
            .context("start schema entry")?;
        writer.write_all(&bytes).context("write schema entry")?;
    }

    let sbom_bytes = serde_cbor::to_vec(sbom).context("encode sbom")?;
    writer
        .start_file("sbom.cbor", FileOptions::default())
        .context("start sbom entry")?;
    writer.write_all(&sbom_bytes).context("write sbom entry")?;

    writer.finish().context("finish zip")?;
    fs::rename(&temp_path, gtpack_out).context("replace gtpack")?;
    Ok(())
}

use std::path::Path;

pub mod movingai;

pub fn walk(base: impl AsRef<Path>, rope: &mut Vec<String>, f: &mut impl FnMut(&Path, &[String])) {
    for entry in base.as_ref().read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        rope.push(path.file_stem().unwrap().to_string_lossy().into_owned());
        let t = entry.file_type().unwrap();
        if t.is_file() {
            f(&path, rope);
        } else if t.is_dir() {
            walk(path, rope, f);
        }
        rope.pop();
    }
}

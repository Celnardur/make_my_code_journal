
pub mod journal {
    pub mod diffs {
        pub struct DiffInfo {
            pub counts: LineCounts,
            pub files: HashMap<String, FileChanges>
        }

        pub struct FileChanges {
            pub counts: LineCounts,
            pub header: String,
            pub path: String,
            pub hunks: Vec<Hunk>,
        }

        pub struct Hunk {
            pub counts: LineCounts,
            pub header: String,
            pub content: String,
        }

        pub struct LineCounts {
            pub added_lines: u64,
            pub deleted_lines: u64,
            pub moddified_lines: u64,
        }
    }
}

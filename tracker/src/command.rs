pub enum TrackerCommand {
    Toggle,
    Add { title: String },
    Remove { uuid: String },
    Select { uuid: String },
    Cut,
}

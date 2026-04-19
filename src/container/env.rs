enum HostEnvVariable {
    Value { value: String },
    Reference { name: String },
}

pub struct EnvRecord {
    name: String,
    host: HostEnvVariable,
}

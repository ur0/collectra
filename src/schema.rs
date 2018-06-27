table! {
    devices (id) {
        id -> Int4,
        udid_sha256 -> Varchar,
        ios_version -> Varchar,
        electra_version -> Int4,
        device_model -> Varchar,
    }
}

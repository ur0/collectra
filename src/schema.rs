table! {
    devices (id) {
        id -> Int4,
        udid -> Varchar,
        ios_version -> Varchar,
        electra_version -> Varchar,
        device_model -> Varchar,
        num_checkins -> Int4,
        last_checkin -> Int8,
    }
}

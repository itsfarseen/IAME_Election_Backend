table! {
    candidates (id) {
        id -> Int4,
        name -> Text,
        school_id -> Int4,
        election_id -> Int4,
        class_id -> Int4,
        gender -> Int4,
        symbol -> Text,
    }
}

table! {
    elections (id) {
        id -> Int4,
        school_id -> Int4,
        name -> Text,
        presidential -> Bool,
        genders -> Int4,
    }
}

table! {
    school_classes (id) {
        id -> Int4,
        school_id -> Int4,
        name -> Text,
        boys -> Int4,
        girls -> Int4,
    }
}

table! {
    schools (id) {
        id -> Int4,
        name -> Text,
        email -> Text,
        password -> Text,
    }
}

table! {
    voted (id) {
        id -> Int4,
        voter_num -> Int4,
        class_id -> Int4,
    }
}

table! {
    votes (id) {
        id -> Int4,
        candidate_id -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    candidates,
    elections,
    school_classes,
    schools,
    voted,
    votes,
);

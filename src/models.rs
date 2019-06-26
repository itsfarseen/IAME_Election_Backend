use crate::schema::*;
use diesel::sql_types::{BigInt, Integer, Text};

pub const GENDER_BOY: i32 = 0;
pub const GENDER_GIRL: i32 = 1;

pub const GENDER_ELECTION_BOYS: i32 = 0;
pub const GENDER_ELECTION_GIRLS: i32 = 1;
pub const GENDER_ELECTION_BOTH: i32 = 2;

#[derive(Insertable, Deserialize)]
#[table_name = "schools"]
pub struct NewSchool {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Queryable, Serialize)]
pub struct School {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Insertable, Deserialize)]
#[table_name = "school_classes"]
pub struct NewSchoolClass {
    pub school_id: i32,
    pub name: String,
    pub boys: i32,
    pub girls: i32,
}

#[derive(Queryable, Serialize)]
pub struct SchoolClass {
    pub id: i32,
    pub school_id: i32,
    pub name: String,
    pub boys: i32,
    pub girls: i32,
}

pub enum ElectionGender {
    All = 0,
    Boys = 1,
    Girls = 2,
}

pub enum Gender {
    Boy = 0,
    Girl = 1,
}

#[derive(Insertable, Deserialize)]
#[table_name = "elections"]
pub struct NewElection {
    pub school_id: i32,
    pub name: String,
    pub presidential: bool,
    pub genders: i32,
}

#[derive(Queryable, Serialize)]
pub struct Election {
    pub id: i32,
    pub school_id: i32,
    pub name: String,
    pub presidential: bool,
    pub genders: i32,
}

#[derive(Insertable, Deserialize, Debug)]
#[table_name = "candidates"]
pub struct NewCandidate {
    pub name: String,
    pub school_id: i32,
    pub class_id: i32,
    pub election_id: i32,
    pub gender: i32,
    pub symbol: String,
}

#[derive(Queryable, Serialize, Debug)]
pub struct Candidate {
    pub id: i32,
    pub name: String,
    pub school_id: i32,
    pub election_id: i32,
    pub class_id: i32,
    pub gender: i32,
    pub symbol: String,
}

#[derive(QueryableByName, Serialize)]
pub struct CandidateResult {
    #[sql_type = "Integer"]
    pub id: i32,
    #[sql_type = "Text"]
    pub name: String,
    #[sql_type = "Integer"]
    pub school_id: i32,
    #[sql_type = "Integer"]
    pub class_id: i32,
    #[sql_type = "Integer"]
    pub election_id: i32,
    #[sql_type = "Integer"]
    pub gender: i32,
    #[sql_type = "BigInt"]
    pub votes: i64,
}

#[derive(Insertable)]
#[table_name = "votes"]
pub struct NewVote {
    pub candidate_id: i32,
}

#[derive(Insertable)]
#[table_name = "voted"]
pub struct NewVoted {
    pub voter_num: i32,
    pub class_id: i32,
}

#[derive(Queryable)]
pub struct Voted {
    pub id: i32,
    pub voter_num: i32,
    pub class_id: i32,
}

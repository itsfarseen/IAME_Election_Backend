#![feature(proc_macro_hygiene, decl_macro, never_type)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate jsonwebtoken as jwt;
use jwt::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::fairing::AdHoc;
use rocket::Rocket;

use std::time::{SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
#[macro_use]
extern crate diesel_migrations;
use diesel_migrations::embed_migrations;

use serde::Serialize;

use rocket_contrib::json::Json;

mod models;
mod schema;
use dotenv;
use models::*;
use schema::*;

#[database("election_db")]
struct Database(diesel::PgConnection);

#[derive(Serialize)]
struct Response<T: Serialize> {
    success: bool,
    message: String,
    data: Option<T>,
}

type JsonResponse<T> = Json<Response<T>>;

#[post("/signup", data = "<school>")]
fn signup(db: Database, school: Json<NewSchool>) -> JsonResponse<School> {
    use schema::schools;
    let school = diesel::insert_into(schools::table)
        .values(&school.0)
        .get_result(&db.0)
        .unwrap();

    Json(Response {
        success: true,
        message: String::from("Saved new school"),
        data: Some(school),
    })
}

#[derive(Deserialize, Queryable)]
struct Login {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginToken {
    school_id: i32,
    exp: usize,
}

impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for LoginToken {
    type Error = String;
    fn from_request(
        request: &'a rocket::request::Request<'r>,
    ) -> rocket::request::Outcome<LoginToken, String> {
        // This closure will execute at most once per request, regardless of
        // the number of times the `User` guard is executed.
        let header_val = request.headers().get_one("Authorization").and_then(|h| {
            let mut iter = h.split(" ");
            iter.next();
            iter.next()
        });
        if header_val.is_some() {
            let header_val = header_val.unwrap();
            let tok = decode(
                &header_val,
                &DecodingKey::from_secret("secret".as_ref()),
                &Validation::default(),
            );
            if tok.is_ok() {
                println!("OK");
                return rocket::Outcome::Success(tok.unwrap().claims);
            } else {
                println!("{}", tok.err().unwrap())
            }
        }
        println!("FAIL");
        return rocket::Outcome::Failure((
            rocket::http::Status::new(401, "Unauthorized"),
            "Unauthorized".to_owned(),
        ));
    }
}

macro_rules! map_err {
    ($l:expr) => {
        match $l {
            Ok(x) => x,
            Err(e) => {
                println!("Debug: {}", e);
                return Json(Response {
                    success: false,
                    message: String::from("Database operation failed."),
                    data: None,
                });
            }
        }
    };
}

#[post("/login", data = "<creds>")]
fn login(db: Database, creds: Json<Login>) -> JsonResponse<String> {
    use schema::schools::dsl::*;
    let school: School = map_err!(schools.filter(email.eq(&creds.0.email)).first(&db.0));
    if school.password == creds.password {
        let claims = LoginToken {
            school_id: school.id,
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize
                + 3600,
        };
        Json(Response {
            success: true,
            message: "Login success".to_owned(),
            data: Some(
                encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret("secret".as_ref()),
                )
                .unwrap(),
            ),
        })
    } else {
        Json(Response {
            success: false,
            message: "Login failed".to_owned(),
            data: None,
        })
    }
}

#[get("/profile")]
fn profile(db: Database, token: LoginToken) -> JsonResponse<School> {
    use schema::schools;
    let school = schools::table
        .filter(schools::id.eq(token.school_id))
        .first(&db.0)
        .unwrap();
    Json(Response {
        success: true,
        message: "Your profile".to_owned(),
        data: Some(school),
    })
}

#[get("/classes")]
fn get_classes(token: LoginToken, db: Database) -> JsonResponse<Vec<SchoolClass>> {
    use schema::school_classes::dsl::*;
    let classes: Vec<SchoolClass> = school_classes
        .filter(school_id.eq(token.school_id))
        .load(&db.0)
        .unwrap();
    Json(Response {
        success: true,
        message: "List of classes for your school.".to_owned(),
        data: Some(classes),
    })
}

#[derive(Deserialize, AsChangeset)]
#[table_name = "school_classes"]
struct SchoolClassInput {
    name: String,
    boys: i32,
    girls: i32,
}

#[post("/classes", data = "<klass>")]
fn add_class(
    token: LoginToken,
    db: Database,
    klass: Json<SchoolClassInput>,
) -> JsonResponse<SchoolClass> {
    use schema::school_classes;
    let school_class = diesel::insert_into(school_classes::table)
        .values(NewSchoolClass {
            school_id: token.school_id,
            name: klass.0.name,
            boys: klass.0.boys,
            girls: klass.0.girls,
        })
        .get_result(&db.0)
        .unwrap();
    Json(Response {
        success: true,
        message: "Added class".to_owned(),
        data: Some(school_class),
    })
}

#[delete("/classes/<class_id>")]
fn delete_class(class_id: i32, token: LoginToken, db: Database) -> JsonResponse<SchoolClass> {
    use schema::school_classes;
    match diesel::delete(school_classes::table)
        .filter(school_classes::id.eq(class_id))
        .filter(school_classes::school_id.eq(token.school_id))
        .get_result(&db.0)
    {
        Ok(school_class) => Json(Response {
            success: true,
            message: "Class deleted".to_owned(),
            data: Some(school_class),
        }),
        Err(_) => Json(Response {
            success: false,
            message: "Couldn't find class".to_owned(),
            data: None,
        }),
    }
}

#[put("/classes/<class_id>", data = "<klass>")]
fn update_class(
    class_id: i32,
    token: LoginToken,
    db: Database,
    klass: Json<SchoolClassInput>,
) -> JsonResponse<SchoolClass> {
    use schema::school_classes;
    match diesel::update(school_classes::table)
        .filter(school_classes::id.eq(class_id))
        .filter(school_classes::school_id.eq(token.school_id))
        .set(&klass.0)
        .get_result(&db.0)
    {
        Ok(school_class) => Json(Response {
            success: true,
            message: "Class updated".to_owned(),
            data: Some(school_class),
        }),
        Err(_) => Json(Response {
            success: false,
            message: "Couldn't find class".to_owned(),
            data: None,
        }),
    }
}

#[get("/elections")]
fn get_elections(token: LoginToken, db: Database) -> JsonResponse<Vec<Election>> {
    use schema::elections::dsl::*;
    let elections_vec = elections
        .filter(school_id.eq(token.school_id))
        .order_by(id)
        .load(&db.0)
        .unwrap();
    Json(Response {
        success: true,
        message: "Elections in your school".to_owned(),
        data: Some(elections_vec),
    })
}

#[derive(Deserialize, AsChangeset)]
#[table_name = "elections"]
struct ElectionInput {
    pub name: String,
    pub presidential: bool,
    pub genders: i32,
}

#[post("/elections", data = "<election>")]
fn add_election(
    token: LoginToken,
    db: Database,
    election: Json<ElectionInput>,
) -> JsonResponse<Election> {
    use schema::elections;
    let election: Election = diesel::insert_into(elections::table)
        .values(NewElection {
            school_id: token.school_id,
            name: election.name.clone(),
            presidential: election.presidential,
            genders: election.genders,
        })
        .get_result(&db.0)
        .unwrap();
    Json(Response {
        success: true,
        message: "Added election".to_owned(),
        data: Some(election),
    })
}

#[delete("/elections/<election_id>")]
fn delete_election(token: LoginToken, db: Database, election_id: i32) -> JsonResponse<Election> {
    use schema::elections;
    match diesel::delete(elections::table)
        .filter(elections::school_id.eq(token.school_id))
        .filter(elections::id.eq(election_id))
        .get_result(&db.0)
    {
        Ok(election) => Json(Response {
            success: true,
            message: "Election deleted".to_owned(),
            data: Some(election),
        }),
        Err(_) => Json(Response {
            success: false,
            message: "Election deletion failed".to_owned(),
            data: None,
        }),
    }
}

#[put("/elections/<election_id>", data = "<election>")]
fn update_election(
    token: LoginToken,
    db: Database,
    election_id: i32,
    election: Json<ElectionInput>,
) -> JsonResponse<Election> {
    use schema::elections;
    match diesel::update(elections::table)
        .filter(elections::school_id.eq(token.school_id))
        .filter(elections::id.eq(election_id))
        .set(&election.0)
        .get_result(&db.0)
    {
        Ok(election) => Json(Response {
            success: true,
            message: "Election updated".to_owned(),
            data: Some(election),
        }),
        Err(_) => Json(Response {
            success: false,
            message: "Election update failed".to_owned(),
            data: None,
        }),
    }
}

#[get("/candidates")]
fn get_candidates(db: Database, token: LoginToken) -> JsonResponse<Vec<Candidate>> {
    use schema::candidates::dsl::*;
    let candidates_vec = candidates
        .filter(school_id.eq(token.school_id))
        .order_by(id)
        .load(&db.0)
        .unwrap();

    Json(Response {
        success: true,
        message: "List of candidates".to_owned(),
        data: Some(candidates_vec),
    })
}

#[derive(Deserialize, AsChangeset, Debug)]
#[table_name = "candidates"]
struct CandidateInput {
    name: String,
    class_id: i32,
    election_id: i32,
    gender: i32,
    symbol: String,
}

#[post("/candidates", data = "<candidate>")]
fn add_candidate(
    db: Database,
    token: LoginToken,
    candidate: Json<CandidateInput>,
) -> JsonResponse<Candidate> {
    use schema::candidates;
    let new_candidate = NewCandidate {
        school_id: token.school_id,
        name: candidate.name.clone(),
        class_id: candidate.class_id,
        election_id: candidate.election_id,
        gender: candidate.gender,
        symbol: candidate.0.symbol,
    };

    let saved_candidate: Candidate = diesel::insert_into(candidates::table)
        .values(new_candidate)
        .get_result(&db.0)
        .unwrap();

    Json(Response {
        success: true,
        message: "Added candidate".to_owned(),
        data: Some(saved_candidate),
    })
}

#[delete("/candidates/<candidate_id>")]
fn delete_candidate(db: Database, token: LoginToken, candidate_id: i32) -> JsonResponse<Candidate> {
    use schema::candidates;
    match diesel::delete(candidates::table)
        .filter(candidates::id.eq(candidate_id))
        .filter(candidates::school_id.eq(token.school_id))
        .get_result(&db.0)
    {
        Ok(candidate) => Json(Response {
            success: true,
            message: "Candidate deleted".to_owned(),
            data: Some(candidate),
        }),
        Err(_) => Json(Response {
            success: false,
            message: "Candidate failed to delete".to_owned(),
            data: None,
        }),
    }
}

#[derive(Deserialize)]
struct VoterInfo {
    student_num: i32,
    class_id: i32,
    school_id: i32,
    gender: i32,
}

#[derive(Queryable, Serialize)]
struct CandidateInfo {
    id: i32,
    election_name: String,
    name: String,
    symbol: String,
}

#[post("/voter/get", data = "<voter>")]
fn get_candidates_for_voter(
    db: Database,
    voter: Json<VoterInfo>,
) -> JsonResponse<Vec<CandidateInfo>> {
    use schema::voted;
    if voted::table
        .filter(voted::voter_num.eq(voter.student_num))
        .filter(voted::class_id.eq(voter.class_id))
        .first::<Voted>(&db.0)
        .is_ok()
    {
        return Json(Response {
            success: false,
            message: "Already voted".to_owned(),
            data: None,
        });
    }

    use schema::candidates;
    use schema::elections;
    let candidates = candidates::table
        .inner_join(elections::table.on(candidates::election_id.eq(elections::id)))
        .select((
            candidates::id,
            elections::name,
            candidates::name,
            candidates::symbol,
        ))
        .filter(candidates::school_id.eq(voter.school_id))
        .filter(
            elections::genders
                .eq(GENDER_ELECTION_BOTH)
                .or(elections::genders.eq(voter.gender)),
        )
        .filter(
            elections::presidential
                .eq(true)
                .or(candidates::class_id.eq(voter.class_id)),
        )
        .load(&db.0)
        .unwrap();
    Json(Response {
        success: true,
        message: "Candidates for given voter".to_owned(),
        data: Some(candidates),
    })
}

#[derive(Deserialize)]
struct VoterVote {
    student_num: i32,
    class_id: i32,
    vote_candidate_ids: Vec<i32>,
}

#[post("/voter/cast", data = "<voter_vote>")]
fn cast_vote(voter_vote: Json<VoterVote>, db: Database) -> JsonResponse<()> {
    use schema::voted;
    if voted::table
        .filter(voted::voter_num.eq(voter_vote.student_num))
        .filter(voted::class_id.eq(voter_vote.class_id))
        .first::<Voted>(&db.0)
        .is_ok()
    {
        return Json(Response {
            success: false,
            message: "Already voted".to_owned(),
            data: None,
        });
    }

    use schema::votes;

    diesel::insert_into(votes::table)
        .values::<Vec<NewVote>>(
            voter_vote
                .vote_candidate_ids
                .iter()
                .map(|&candidate_id| NewVote { candidate_id })
                .collect(),
        )
        .execute(&db.0)
        .unwrap();
    diesel::insert_into(voted::table)
        .values(NewVoted {
            voter_num: voter_vote.student_num,
            class_id: voter_vote.class_id,
        })
        .execute(&db.0)
        .unwrap();
    Json(Response {
        success: true,
        message: "Voted successfully".to_owned(),
        data: Some(()),
    })
}

#[get("/result")]
fn result(token: LoginToken, db: Database) -> JsonResponse<Vec<CandidateResult>> {
    let results = diesel::sql_query(
        format!("SELECT candidates.id, name, school_id, class_id, election_id, gender, COUNT(votes.id) as votes FROM candidates LEFT JOIN votes on candidates.id = votes.candidate_id WHERE school_id={} GROUP BY candidates.id ", token.school_id)
    )
    .load(&db.0).unwrap();
    Json(Response {
        success: true,
        message: "Results".to_owned(),
        data: Some(results),
    })
}

#[post("/delete_votes")]
fn delete_votes(token: LoginToken, db: Database) -> JsonResponse<()> {
    use schema::candidates;
    use schema::school_classes;
    use schema::votes;
    diesel::delete(votes::table)
        .filter(
            votes::candidate_id.eq_any(
                candidates::table
                    .select(candidates::id)
                    .filter(candidates::school_id.eq(token.school_id)),
            ),
        )
        .execute(&db.0)
        .unwrap();
    diesel::delete(voted::table)
        .filter(
            voted::class_id.eq_any(
                school_classes::table
                    .select(school_classes::id)
                    .filter(school_classes::school_id.eq(token.school_id)),
            ),
        )
        .execute(&db.0)
        .unwrap();
    Json(Response {
        success: true,
        message: "Votes cleared for your school".to_owned(),
        data: None,
    })
}

embed_migrations!();

fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = Database::get_one(&rocket).expect("database connection");
    match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            panic!("Failed to run database migrations: {:?}", e);
        }
    }
}

fn main() {
    dotenv::dotenv().unwrap();
    rocket::ignite()
        .attach(Database::fairing())
        .attach(AdHoc::on_attach("DB Migrations", run_db_migrations))
        .mount(
            "/",
            routes![
                signup,
                login,
                profile,
                get_classes,
                add_class,
                delete_class,
                update_class,
                get_elections,
                add_election,
                delete_election,
                update_election,
                get_candidates,
                add_candidate,
                delete_candidate,
                get_candidates_for_voter,
                cast_vote,
                result,
                delete_votes
            ],
        )
        .launch();
}

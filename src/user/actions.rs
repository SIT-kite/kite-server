use diesel::{PgConnection, RunQueryDsl, ExpressionMethods, QueryDsl};
use crate::user::error::Result;
use crate::user::error::OperationError::{Conflict, NoSuchRecord, NoMoreVerification};
use crate::user::error::UserError::OpError;
use crate::user::models::{Person, Verification};
use crate::schema;


// Require to verify the credential and login.
// The function will return an error::Result. When the process success, an i32 value as uid
// will be returned. Otherwise, a UserError enum, provides the reason.
pub fn login(conn: &PgConnection, auth: Verification) -> Result<i32> {
    use schema::verifications::dsl::*;

    // Considering that some administrators or users may use multiple WeChat accounts or
    // user names, one person binds two or more similar verifications is supported.
    let results = verifications
        .filter(login_type.eq(auth.login_type))
        .filter(account.eq(auth.account.clone()))
        .load::<Verification>(conn)?;

    // Try each credential to check.
    // Normally, certain accounts will not bind the same authentication way, but
    // when it really occurred, the function will return the first account.
    for each_verification_way in results {
        if each_verification_way.credential == auth.credential {
            return Ok(each_verification_way.uid);
        }
    }
    // Return no such user found.
    Err(OpError(NoSuchRecord))
}

fn get_bound_count(conn: &PgConnection, auth: &Verification) -> Result<i64> {
    use schema::verifications::dsl::*;

    let count = verifications
        .count()
        .filter(login_type.eq(auth.login_type))
        .filter(account.eq(auth.account.clone()))
        .get_result::<i64>(conn)?;
    Ok(count)
}

// Bind some verification approaches like WeChat or username/password to some person's
// account. The method will check if the credential already bound, and guarantee a ghost
// don't access to the system. Once the process success, the Verification itself will be
// returned.
pub fn bind(conn: &PgConnection, auth: Verification) -> Result<Verification> {
    use schema::verifications::dsl::*;

    // To avoid one verification approach bind more than one person.
    // Maybe it can be changed in later version :D
    if get_bound_count(conn, &auth)? > 0 {
        return Err(OpError(Conflict));
    }
    diesel::insert_into(verifications)
        .values(&auth)
        .execute(conn)?;
    Ok(auth)
}

// Unbind verification approach with one person. Note that one person must bind at least one
// verification way.
pub fn unbind(conn: &PgConnection, auth: Verification) -> Result<()> {
    use schema::verifications::dsl::*;

    let uid_bindings_count = verifications.filter(uid.eq(auth.uid))
        .count()
        .get_result::<i64>(conn)?;
    if uid_bindings_count < 2 {
        return Err(OpError(NoMoreVerification));
    }
    let affected_rows = diesel::delete(verifications
        .filter(login_type.eq(auth.login_type))
        .filter(account.eq(auth.account.clone()))
    ).execute(conn)?;
    if affected_rows == 0 {
        return Err(OpError(NoSuchRecord));
    }
    Ok(())
}

pub fn search_by_uid(conn: &PgConnection, uid: i32) -> Result<Vec<Verification>> {
    use schema::verifications::dsl;

    let results = dsl::verifications.filter(dsl::uid.eq(uid))
        .get_results::<Verification>(conn)?;
    Ok(results)
}


pub fn is_uid_existed(conn: &PgConnection, uid: i32) -> Result<bool> {
    use schema::persons::dsl;

    let count = dsl::persons
        .filter(dsl::uid.eq(uid))
        .count()
        .get_result::<i64>(conn)?;
    Ok(count > 0)
}

pub fn create(conn: &PgConnection, person: Person) -> Result<Person> {
    use schema::persons::dsl;

    let new_person = diesel::insert_into(dsl::persons)
        .values(person)
        .get_result::<Person>(conn)?;
    Ok(new_person)
}

pub fn delete_user_by_uid(conn: &PgConnection, uid: i32) -> Result<()> {
    use schema::persons::dsl;

    let affected_rows = diesel::delete(dsl::persons
        .filter(dsl::uid.eq(uid))).execute(conn)?;
    if affected_rows != 1 {
        return Err(OpError(NoSuchRecord));
    }
    Ok(())
}

pub fn delete_user(conn: &PgConnection, person: Person) -> Result<()> {
    delete_user_by_uid(conn, person.uid)
}

pub fn get_person(conn: &PgConnection, uid: i32) -> Result<Person> {
    use schema::persons::dsl;

    let result = dsl::persons
        .filter(dsl::uid.eq(uid))
        .get_result::<Person>(conn)?;
    Ok(result)
}

pub fn list(conn: &PgConnection, page_index: i64, page_size: i32) -> Result<Vec<Person>> {
    use schema::persons::dsl;

    let page_index = if page_index < 1 { 1 } else { page_index };
    let page_size = if page_size < 1 { 10 } else { page_size };
    let results = dsl::persons
        .offset((page_index - 1) * page_size as i64)
        .limit(page_size.into())
        .get_results::<Person>(conn)?;
    Ok(results)
}


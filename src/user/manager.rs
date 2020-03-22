use super::*;
use models::*;
use error::Result;
use crate::user::error::UserError::OpError;
use crate::user::error::OperationError::{UserNotFound, Conflict, NoMoreVerification, NoSuchVerification};
use crate::user::models::{Verification, Person};
use std::panic::resume_unwind;


impl Verification {
    // Require to verify the credential and login.
    // The function will return an error::Result. When the process success, an i32 value as uid
    // will be returned. Otherwise, a UserError enum, provides the reason.
    pub fn login(&self) -> Result<i32> {
        use schema::verifications::dsl::*;
        let connection = establish_connection();

        // Considering that some administrators or users may use multiple WeChat accounts or
        // user names, one person binds two or more similar verifications is supported.
        let results = verifications
            .filter(login_type.eq(self.login_type))
            .filter(account.eq(self.account.clone()))
            .load::<Verification>(&connection)?;

        // Try each credential to authenticate.
        for each_verification_way in results {
            if each_verification_way.credential == self.credential {
                return Ok(each_verification_way.uid);
            }
        }
        // Return no such user found.
        Err(OpError(UserNotFound))
    }

    fn get_bound_count(&self) -> Result<i64> {
        use schema::verifications::dsl::*;

        let connection = establish_connection();
        let count = verifications
            .count()
            .filter(login_type.eq(self.login_type))
            .filter(account.eq(self.account.clone()))
            .get_result::<i64>(&connection)?;
        Ok(count)
    }

    // Bind some verification approaches like WeChat or username/password to some person's
    // account. The method will check if the credential already bound, and guarantee a ghost
    // don't access to the system. Once the process success, the Verification itself will be
    // returned.
    pub fn bind(&self) -> Result<&Self> {
        use schema::verifications::dsl::*;
        let connection = establish_connection();

        // To avoid one verification approach bind more than one person.
        // Maybe it can be changed in later version :D
        if self.get_bound_count()? > 0 {
            return Err(OpError(Conflict));
        }
        diesel::insert_into(verifications)
            .values(self)
            .execute(&connection)?;
        Ok(self)
    }

    // Unbind verification approach with one person. Note that one person must bind at least one
    // verification way.
    pub fn unbind(&self) -> Result<()> {
        use schema::verifications::dsl::*;
        let connection = establish_connection();
        let uid_bindings_count = verifications.filter(uid.eq(self.uid))
            .count()
            .get_result::<i64>(&connection)?;
        if uid_bindings_count < 2 {
            return Err(OpError(NoMoreVerification));
        }
        let affected_rows = diesel::delete(verifications
            .filter(login_type.eq(self.login_type))
            .filter(account.eq(self.account.clone()))
        )
            .execute(&connection)?;
        if affected_rows == 0 {
            return Err(OpError(NoSuchVerification));
        }
        Ok(())
    }

    pub fn search_by_uid(uid: i32) -> Result<Vec<Verification>> {
        use schema::verifications::dsl;
        let connection = establish_connection();
        let results = dsl::verifications.filter(dsl::uid.eq(uid))
            .get_results::<Verification>(&connection)?;
        Ok(results)
    }

}




impl Person {
    pub fn is_uid_existed(uid: i32) -> Result<bool> {
        use schema::persons::dsl;
        let connection = establish_connection();

        let count = dsl::persons
            .filter(dsl::uid.eq(uid))
            .count()
            .get_result::<i64>(&connection)?;
        Ok(count > 0)
    }

    pub fn create(&self) -> Result<Self> {
        use schema::persons::dsl;
        let connection = establish_connection();

        let new_person = diesel::insert_into(dsl::persons)
            .values(self)
            .get_result::<Person>(&connection)?;
        Ok(new_person)
    }

    pub fn delete_by_uid(uid: i32) -> Result<()> {
        use schema::persons::dsl;
        let connection = establish_connection();

        let affected_rows = diesel::delete(dsl::persons
            .filter(dsl::uid.eq(uid))).execute(&connection)?;
        if affected_rows != 1 {
            return Err(OpError(UserNotFound));
        }
        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        Person::delete_by_uid(self.uid)
    }

    pub fn get(uid: i32) -> Result<Self> {
        use schema::persons::dsl;
        let connection = establish_connection();

        let result = dsl::persons
            .filter(dsl::uid.eq(uid))
            .get_result::<Person>(&connection)?;
        Ok(result)
    }

    pub fn list(page_index: i64, page_size: i32) -> Result<Vec<Self>> {
        use schema::persons::dsl;
        let connection = establish_connection();

        let page_index = if page_index < 1 { 1 } else { page_index };
        let page_size = if page_size < 1 { 10 } else { page_size };
        let results = dsl::persons
            .offset((page_index - 1) * page_size as i64)
            .limit(page_size.into())
            .get_results::<Person>(&connection)?;
        Ok(results)
    }
//    pub fn update() -> Result<Self> {
//
//    }

}

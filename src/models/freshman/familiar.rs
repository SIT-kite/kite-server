use super::{FreshmanBasic, NewMate, PeopleFamiliar};
use crate::error::Result;
use crate::models::freshman::{FreshmanAnalysis, MapDefaultAvatar};
use sqlx::{postgres::PgQueryAs, PgPool};

impl FreshmanBasic {
    /* Classmates, roommates, and familiar people */

    /// Get classmate.
    pub async fn get_classmates(&self, client: &PgPool) -> Result<Vec<NewMate>> {
        let classmates: Vec<NewMate> = sqlx::query_as(
            "SELECT college, major, name, stu.province, building, room, bed, last_seen, avatar, contact
            FROM freshman.students AS stu
            LEFT JOIN public.person AS person
            ON stu.uid = person.uid
            INNER JOIN (
                    SELECT class FROM freshman.students WHERE student_id = $1 LIMIT 1
                ) self
            ON
                stu.class = self.class
                AND stu.student_id <> $1
            ORDER BY stu.student_id",
        )
        .bind(&self.student_id)
        .fetch_all(client)
        .await?;

        Ok(classmates.map_default_avatar())
    }

    pub async fn get_roommates(&self, client: &PgPool) -> Result<Vec<NewMate>> {
        let roommates: Vec<NewMate> = sqlx::query_as(
            "SELECT college, major, name, stu.province, stu.building, stu.room, bed, last_seen, avatar, contact
            FROM freshman.students AS stu
            LEFT JOIN public.person AS person
            ON stu.uid = person.uid
            INNER JOIN (
                    SELECT building, room FROM freshman.students WHERE student_id = $1 LIMIT 1
                ) self
            ON
                stu.room = self.room
                AND stu.building = self.building
                AND stu.student_id <> $1",
        )
        .bind(&self.student_id)
        .fetch_all(client)
        .await?;

        Ok(roommates.map_default_avatar())
    }

    pub async fn get_people_familiar(&self, client: &PgPool) -> Result<Vec<PeopleFamiliar>> {
        let people_familiar: Vec<PeopleFamiliar> = sqlx::query_as(
            "SELECT DISTINCT name, college, stu.city, last_seen, avatar, contact
            FROM freshman.students AS stu
            LEFT JOIN public.person AS person
            ON stu.uid = person.uid
            INNER JOIN (
                    SELECT graduated_from, city, postcode FROM freshman.students WHERE student_id = $1 LIMIT 1
                ) self
            ON
                ((stu.graduated_from = self.graduated_from)
                OR stu.city = self.city
                OR stu.postcode / 1000 = self.postcode / 1000)
                AND stu.visible = true
                AND stu.student_id <> $1",
        )
            .bind(&self.student_id)
            .fetch_all(client)
            .await?;

        Ok(people_familiar.map_default_avatar())
    }

    /* Get statistics */

    /// Get analysis data.
    // TODO: use SQL procedure to speed up the query.
    pub async fn get_analysis(&self, pool: &PgPool) -> Result<FreshmanAnalysis> {
        use super::GenderAnalysis;

        let gender_count = self.count_by_gender(pool).await?;
        Ok(FreshmanAnalysis {
            same_name: self.count_by_name(pool).await?,
            same_city: self.count_by_city(pool).await?,
            same_high_school: self.count_by_graduated_school(pool).await?,
            college_count: self.count_by_college(pool).await?,
            major: GenderAnalysis {
                total: gender_count.0 + gender_count.1,
                boys: gender_count.0,
                girls: gender_count.1,
            },
        })
    }

    /// Count the number of people with the same name
    async fn count_by_name(&self, client: &PgPool) -> Result<i64> {
        let same_name_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) - 1 FROM freshman.students WHERE student_id = $1")
                .bind(&self.student_id)
                .fetch_one(client)
                .await?;
        Ok(same_name_count.0)
    }

    /// Count the freshman in the given major.
    async fn count_by_major(&self, client: &PgPool) -> Result<i64> {
        let major_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM freshman.students WHERE major = $1")
                .bind(&self.major)
                .fetch_one(client)
                .await?;
        Ok(major_count.0)
    }

    /// Count the freshman in the given college.
    async fn count_by_college(&self, client: &PgPool) -> Result<i64> {
        let college_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM freshman.students WHERE college = $1")
                .bind(&self.college)
                .fetch_one(client)
                .await?;
        Ok(college_count.0)
    }

    /// Count the freshman of boy and girl in the current major.
    async fn count_by_gender(&self, client: &PgPool) -> Result<(i64, i64)> {
        let result: (i64, i64, ) =
            sqlx::query_as(
                "SELECT
                        (SELECT COUNT(student_id) FROM freshman.students WHERE major = $1 AND gender = 'M') 
                        AS major_boy_count, 
                        (SELECT COUNT(student_id) FROM freshman.students WHERE major = $1 AND gender = 'F') 
                        AS major_girl_count")
                .bind(&self.major)
                .fetch_one(client)
                .await?;
        Ok((result.0, result.1))
    }

    async fn count_by_graduated_school(&self, client: &PgPool) -> Result<i64> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(student_id) - 1 FROM freshman.students
                    WHERE graduated_from = 
                        (SELECT graduated_from FROM freshman.students WHERE student_id = $1)",
        )
        .bind(&self.student_id)
        .fetch_one(client)
        .await?;
        Ok(result.0)
    }

    async fn count_by_city(&self, client: &PgPool) -> Result<i64> {
        let result: (i64,) = sqlx::query_as(
            "WITH self AS (SELECT city, postcode FROM freshman.students WHERE student_id = $1)
                SELECT COUNT(student_id) - 1 FROM freshman.students s, self
                WHERE s.city = self.city OR s.postcode / 1000 = self.postcode / 1000",
        )
        .bind(&self.student_id)
        .fetch_one(client)
        .await?;
        Ok(result.0)
    }
}

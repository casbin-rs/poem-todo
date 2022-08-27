// Copyright 2022 The casbin Authors. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::todos::{self, dsl::*};
use crate::users;
use diesel::prelude::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};

#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(
    Identifiable, Queryable, Associations, Serialize, Deserialize, PartialEq, Eq, Debug, Default,
)]
#[diesel(belongs_to(User))]
#[diesel(table_name = todos)]
pub struct Todo {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub finished: bool,
}

#[derive(Insertable, Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = todos)]
pub struct NewTodo {
    pub user_id: i32,
    pub title: String,
    pub finished: bool,
}

impl Todo {
    pub fn find_all(conn: &mut PgConnection) -> QueryResult<Vec<Todo>> {
        todos::table.order(todos::id.asc()).load::<Todo>(conn)
    }

    pub fn find_by_id(i: i32, user: &User, conn: &mut PgConnection) -> QueryResult<Todo> {
        if user.is_admin {
            todos::table.find(i).get_result::<Todo>(conn)
        } else {
            // user can only access his todos
            todos::table
                .find(i)
                .filter(todos::user_id.eq(user.id))
                .get_result::<Todo>(conn)
        }
    }

    pub fn insert(new_todo: NewTodo, conn: &mut PgConnection) -> QueryResult<Todo> {
        let todo_id = diesel::insert_into(todos)
            .values(&new_todo)
            .returning(todos::id)
            .execute(conn);

        if let Err(err) = todo_id {
            return Err(err);
        }
        todos::table
            .find(todo_id.unwrap() as i32)
            .get_result::<Todo>(conn)
    }

    pub fn update(i: i32, update_todo: NewTodo, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::update(todos.find(i))
            .set(&update_todo)
            .execute(conn)
    }

    pub fn delete(delete_id: i32, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::delete(todos.filter(todos::id.eq(delete_id))).execute(conn)
    }
}

impl User {
    pub fn find_all(conn: &mut PgConnection) -> QueryResult<Vec<User>> {
        users::table.order(users::id.asc()).load::<User>(conn)
    }

    pub fn find_by_id(i: i32, conn: &mut PgConnection) -> QueryResult<User> {
        users::table.find(i).get_result::<User>(conn)
    }

    pub fn find_by_name(name: String, conn: &mut PgConnection) -> Result<Option<User>, Error> {
        users::table
            .filter(users::name.eq(name))
            .first::<User>(conn)
            .optional()
    }

    pub fn find_user(
        conn: &mut PgConnection,
        username: &str,
        password: &str,
    ) -> Result<Option<User>, Error> {
        users::table
            .filter(users::name.eq(username))
            .filter(users::password.eq(password))
            .first::<User>(conn)
            .optional()
    }

    pub fn find_self_todos(i: i32, conn: &mut PgConnection) -> QueryResult<Vec<Todo>> {
        let user = users::table.find(i).get_result::<User>(conn)?;
        Todo::belonging_to(&user).load::<Todo>(conn)
    }
}

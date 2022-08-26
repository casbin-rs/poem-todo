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

use crate::User;
use diesel::PgConnection;
use poem::{
    http::StatusCode,
    web::{
        headers,
        headers::{authorization::Basic, HeaderMapExt},
    },
    Endpoint, Error, Middleware, Request, Result,
};
use poem_casbin_auth::CasbinVals;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

pub struct BasicAuth;

impl<E: Endpoint> Middleware<E> for BasicAuth {
    type Output = BasicAuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        BasicAuthEndpoint { ep }
    }
}

pub struct BasicAuthEndpoint<E> {
    ep: E,
}

#[poem::async_trait]
impl<E: Endpoint> Endpoint for BasicAuthEndpoint<E> {
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        if let Some(auth) = req.headers().typed_get::<headers::Authorization<Basic>>() {
            let conn = req.extensions().get::<Arc<Mutex<PgConnection>>>();
            let res = User::find_user(
                conn.unwrap().lock().unwrap().deref_mut(),
                auth.username(),
                auth.password(),
            );
            if res.is_err() {
                return Err(Error::from_status(StatusCode::UNAUTHORIZED));
            }
            let user = res.unwrap();
            return if let Some(user) = user {
                let vals = CasbinVals {
                    subject: String::from(&user.name),
                    domain: None,
                };
                req.extensions_mut().insert(vals);
                req.extensions_mut().insert(user);
                self.ep.call(req).await
            } else {
                Err(Error::from_status(StatusCode::UNAUTHORIZED))
            };
        }
        Err(Error::from_status(StatusCode::UNAUTHORIZED))
    }
}

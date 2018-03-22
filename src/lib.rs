#[macro_use]
extern crate exonum;
extern crate exonum_time;
extern crate iron;

extern crate router;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate byteorder;
extern crate rand;

mod schema;

/// Некоторый уникальный идентификатор сервиса.
pub const CRYPTOOWLS_SERVICE_ID: u16 = 521;
/// Уникальное имя сервиса, которое будет использоваться в апи и конфигурации.
pub const CRYPTOOWLS_SERVICE_NAME: &str = "cryptoowls";

/// Сумма пополнения баланса
pub const ISSUE_AMMOUNT: u64 = 100;

/// Таймаут, после которого разрешено повторное пополнение баланса.
pub const ISSUE_TIMEOUT: u64 = 5 * 60;

/// Таймаут, после которого разрешено повторное размножение.
pub const BREEDING_TIMEOUT: u64 = 5 * 60;

/// Стоимость размножения
pub const BREEDING_PRICE: u64 = 42;


/// Модуль со структурами данных, которые хранятся в блокчейне
mod data_layout {

    use std::time::SystemTime;
    use exonum::crypto::{Hash, PublicKey};

    use byteorder::{ReadBytesExt, BigEndian};

    use std::io::Cursor;
    use rand::{Rng, IsaacRng, SeedableRng};
    use rand::distributions::{Weighted, WeightedChoice, Sample};

    encoding_struct! {
        /// Интересующая нас криптосова, ее уникальный идентифицатор вычисляется как хеш
        /// от этой структуры данных.
        struct CryptoOwl {
            /// Имя совы (должно быть уникальным)
            name: &str,
            /// Генетический код криптосовы.
            dna: u32,
        }
    }

    encoding_struct! {
        /// Текущее состоянии криптосовы
        struct CryptoOwlState {
            /// Сама сова
            owl: CryptoOwl,
            /// Владелец.
            owner: &PublicKey,
            /// Время последнего разведения.
            last_breeding: SystemTime,
        }
    }

    encoding_struct! {
        /// Данные о пользователи и его совах
        struct User {
            /// Его публичный ключ
            public_key: &PublicKey,
            /// Его имя
            name: &str,
            /// Текущий баланс
            balance: u64,
            /// Время последнего пополнения баланса
            last_fillup: SystemTime,
        }
    }

    encoding_struct! {
        /// Ордер на покупку совы
        struct Order {
            /// Тот, кто создал ордер
            public_key: &PublicKey,
            /// Идентификатор совы
            owl_id: &Hash,
            /// pending - в ожидании, accepted - исполнен, declined - отвергнут.
            status: &str,
            /// Цена на сову
            price: u64,
        }
    }

    impl CryptoOwl {
        pub fn breed(&self, other: &CryptoOwl, name: &str, hash_seed: &Hash) -> CryptoOwl {

            let hash_seed: &[u8] = hash_seed.as_ref();
            let mut seed = [0u32; 4];
            let mut cursor = Cursor::new(hash_seed);

            for i in 0..4 {
                seed[i] = cursor.read_u32::<BigEndian>().unwrap();
            }
            let mut rng = IsaacRng::from_seed(&seed);
            let mut son_dna = 0u32;

            for i in 0..32 {
                let mask = 2u32.pow(i);
                let (fg, mg) = (self.dna() & mask, other.dna() & mask);
                if fg == mg {
                    // Если биты у родителей совпадают, то с вероятностью
                    // 8/10 бит ребенка будет таким же
                    let mut possible_genes = vec![
                        Weighted {
                            weight: 8,
                            item: fg,
                        },
                        Weighted {
                            weight: 2,
                            item: fg ^ mask,
                        },
                    ];

                    let mut choices = WeightedChoice::new(&mut possible_genes);
                    son_dna |= choices.sample(&mut rng);

                } else {
                    // Если биты различаются, то результирующий бит будет
                    // выбираться с вероятностью 1/2.
                    if rng.gen() {
                        son_dna |= mask;
                    }
                }
            }

            CryptoOwl::new(name, son_dna)
        }
    }

}

/// Модуль с описанием транзакций для демки.
pub mod transactions {
    use exonum::crypto::{Hash, PublicKey, CryptoHash};
    use exonum::blockchain::{Transaction, ExecutionResult, Schema};
    use exonum::storage::Fork;
    use exonum::messages::Message;

    use schema;
    use data_layout::{User, CryptoOwlState, Order};
    use exonum_time::TimeSchema;

    use std::time::SystemTime;

    use {CRYPTOOWLS_SERVICE_ID, ISSUE_AMMOUNT, ISSUE_TIMEOUT, BREEDING_TIMEOUT, BREEDING_PRICE};

    transactions! {
        pub Transactions {
            const SERVICE_ID = CRYPTOOWLS_SERVICE_ID;
            /// Транзакция создания пользователя
            struct CreateUser {
                /// Публичный идентификатор пользователя
                public_key: &PublicKey,
                /// Имя
                name: &str,
            }
            /// Транзакция создания совы. Если идентификаторы отца и матери это нули,
            /// то выводится базовая сова
            struct MakeOwl {
                /// Публичный идентификатор пользователя
                public_key: &PublicKey,
                /// Имя совенка
                name: &str,
                /// Идентификатор отца
                father_id: &Hash,
                /// Идентификатор матери
                mother_id: &Hash,
            }
            /// Транзакция запроса новых средств
            struct Issue {
                /// Публичный идентификатор пользователя
                public_key: &PublicKey,
                /// Текущее время пользователя (нужно только для обхода replay защиты)
                current_time: SystemTime,
            }
            /// Транзакция размещения нового ордера
            struct CreateOrder
            {
                /// Публичный идентификатор пользователя
                public_key: &PublicKey,
                /// Идентификатор совы
                owl_id: &Hash,
                /// Желаемая цена
                price: u64,
                /// Текущее время пользователя
                current_time: SystemTime,
            }
            /// Одобрение ордера на покупку
            struct AcceptOrder
            {
                /// Публичный идентификатор пользователя
                public_key: &PublicKey,
                /// Идентификатор ордера
                order_id: &Hash,
            }
        }
    }

    impl Transaction for CreateUser {
        fn verify(&self) -> bool {
            self.verify_signature(self.public_key())
        }

        fn execute(&self, fork: &mut Fork) -> ExecutionResult {
            let ts = {
                let time_schema = TimeSchema::new(&fork);
                time_schema.time().get().unwrap()
            };

            let key = self.public_key();

            let mut schema = schema::CryptoOwlsSchema::new(fork);
            if schema.users_proof().get(key).is_none() {
                let user = User::new(&key, self.name(), ISSUE_AMMOUNT, ts);
                schema.users().put(key, user);
            }
            Ok(())
        }
    }


    impl Transaction for MakeOwl {
        fn verify(&self) -> bool {
            self.verify_signature(self.public_key())
        }

        fn execute(&self, fork: &mut Fork) -> ExecutionResult {
            let ts = {
                let time_schema = TimeSchema::new(&fork);
                time_schema.time().get().unwrap()
            };

            let state_hash = {
                let info_schema = Schema::new(&fork);
                info_schema.state_hash_aggregator().root_hash()
            };

            let mut schema = schema::CryptoOwlsSchema::new(fork);
            let parents = [self.mother_id(), self.father_id()]
                .iter()
                .map(|&i| schema.owls_state().get(&i))
                .collect::<Option<Vec<CryptoOwlState>>>();

            let user = schema.users().get(self.public_key()).unwrap();
            let key = user.public_key();

            if let Some(parents) = parents {
                if user.balance() >= BREEDING_PRICE &&
                    parents.iter().all(|ref p| {
                        ts.duration_since(p.last_breeding()).unwrap().as_secs() >= BREEDING_TIMEOUT
                    })
                {
                    let (mother, father) = (parents[0].owl(), parents[1].owl());

                    let son = mother.breed(&father, self.name(), &state_hash);

                    let owl_key = son.hash();
                    let sons_state = CryptoOwlState::new(son, &key, ts);

                    //TODO: add renew_breeding_time method

                    let mothers_state = CryptoOwlState::new(mother, &key, ts);

                    let fathers_state = CryptoOwlState::new(father, &key, ts);

                    let user = User::new(&key, user.name(), user.balance() - BREEDING_PRICE, ts);

                    schema.owls_state().put(&owl_key, sons_state);
                    schema.owls_state().put(self.mother_id(), mothers_state);
                    schema.owls_state().put(self.father_id(), fathers_state);
                    schema.users().put(&key, user);
                }
            }

            Ok(())
        }
    }

    impl Transaction for Issue {
        fn verify(&self) -> bool {
            self.verify_signature(self.public_key())
        }

        fn execute(&self, fork: &mut Fork) -> ExecutionResult {
            let ts = {
                let time_schema = TimeSchema::new(&fork);
                time_schema.time().get().unwrap()
            };

            let mut schema = schema::CryptoOwlsSchema::new(fork);
            let key = self.public_key();
            let user = schema.users().get(key).unwrap();

            if ts.duration_since(user.last_fillup()).unwrap().as_secs() >= ISSUE_TIMEOUT {
                let user = User::new(&key, user.name(), user.balance() + ISSUE_AMMOUNT, ts);
                schema.users().put(&key, user);
            }

            Ok(())
        }
    }

    impl Transaction for CreateOrder {
        fn verify(&self) -> bool {
            self.verify_signature(self.public_key())
        }

        fn execute(&self, fork: &mut Fork) -> ExecutionResult {
            let mut schema = schema::CryptoOwlsSchema::new(fork);
            let key = self.public_key();
            let user = schema.users().get(&key).unwrap();
            if let Some(_) = schema.owls_state().get(self.owl_id()) {
                if self.price() <= user.balance() {
                    let order = Order::new(&key, self.owl_id(), "pending", self.price());
                    let order_hash = order.hash();
                    schema.orders().put(&order.hash(), order);
                    schema.user_orders(&key).push(order_hash);
                    schema.owl_orders(&self.owl_id()).push(order_hash);
                }
            }

            Ok(())
        }
    }

    impl Transaction for AcceptOrder {
        fn verify(&self) -> bool {
            self.verify_signature(self.public_key())
        }

        fn execute(&self, fork: &mut Fork) -> ExecutionResult {
            let mut schema = schema::CryptoOwlsSchema::new(fork);
            if let Some(order) = schema.orders().get(self.order_id()) {
                let buyer = schema.users().get(order.public_key()).unwrap();
                if order.status() == "pending" {
                    if buyer.balance() >= order.price() &&
                        schema.users_owls(self.public_key()).contains(
                            order.owl_id(),
                        )
                    {
                        let new_order = Order::new(
                            order.public_key(),
                            order.owl_id(),
                            "accepted",
                            order.price(),
                        );
                        let owl_state = schema.owls_state().get(order.owl_id()).unwrap();

                        let new_owl_state = CryptoOwlState::new(
                            owl_state.owl(),
                            order.public_key(),
                            owl_state.last_breeding(),
                        );


                        schema.users_owls(self.public_key()).remove(order.owl_id());
                        schema.users_owls(order.public_key()).insert(
                            *order.owl_id(),
                        );

                        schema.orders().put(&order.hash(), new_order);
                        schema.owls_state().put(order.owl_id(), new_owl_state);

                    } else {
                        let new_order = Order::new(
                            order.public_key(),
                            order.owl_id(),
                            "declined",
                            order.price(),
                        );
                        schema.orders().put(&order.hash(), new_order);
                    }
                }
            }
            Ok(())
        }
    }

}

/// Модуль с реализацией api
mod api {
    use iron::prelude::*;
    use iron::Handler;
    use iron::status::Status;
    use iron::headers::ContentType;
    use iron::modifiers::Header;

    use router::Router;


    use exonum::api::{Api, ApiError};
    use exonum::crypto::{Hash, PublicKey};
    use exonum::encoding::serialize::{FromHex, FromHexError};

    use exonum::node::{TransactionSend, ApiSender};
    use exonum::blockchain::{Blockchain, Service, Transaction, ApiContext, ExecutionResult,
                             TransactionSet};

    use schema;
    use data_layout::{CryptoOwlState, User, Order};
    use serde_json;

    #[derive(Clone)]
    struct CryptoOwlsApi {
        channel: ApiSender,
        blockchain: Blockchain,
    }

    impl Api for CryptoOwlsApi {
        fn wire(&self, router: &mut Router) {
            let self_ = self.clone();
            let get_user = move |req: &mut Request| self_.get_user(req);

            let self_ = self.clone();
            let get_users = move |req: &mut Request| self_.get_users(req);

            let self_ = self.clone();
            let get_users_orders = move |req: &mut Request| self_.get_users_orders(req);

            let self_ = self.clone();
            let get_owl = move |req: &mut Request| self_.get_owl(req);

            let self_ = self.clone();
            let get_owls = move |req: &mut Request| self_.get_owls(req);

            let self_ = self.clone();
            let get_owls_orders = move |req: &mut Request| self_.get_owls_orders(req);

            let self_ = self.clone();
            let get_users_owls = move |req: &mut Request| self_.get_users_owls(req);

            // let self_ = self.clone();
            // let post_order = move |req: &mut Request| self_.post_owl_order(req);

            // let self_ = self.clone();
            // let post_acceptance = move |req: &mut Request| self_.post_acceptance(req);

            // Bind handlers to specific routes.

            router.get("/v1/users", get_users, "get_users");
            router.get("/v1/user/:pub_key", get_user, "get_user");

            router.get(
                "/v1/user/:pub_key/orders",
                get_users_orders,
                "get_users_orders",
            );

            router.get("/v1/user/:pub_key/owls", get_users_owls, "get_users_owls");

            router.get("/v1/owl/:owl_hash", get_owl, "get_owl");
            router.get("/v1/owls", get_owls, "get_owls");

            router.get(
                "/v1/owl/:owl_hash/orders",
                get_owls_orders,
                "get_owls_orders",
            );

            // router.post("/v1/order", post_owl_order, "post_owl_order");
            // router.post("/v1/order/:order_hash", post_acceptance, "post_acceptance");
        }
    }

    impl CryptoOwlsApi {
        // /// Вычленение хэша совы из url
        fn find_owl_hash(req: &mut Request) -> Result<Hash, FromHexError> {
            let ref owl_hash = req.extensions
                .get::<Router>()
                .unwrap()
                .find("owl_hash")
                .unwrap();
            Hash::from_hex(owl_hash)
        }

        /// Вычленение публичного ключа из url
        fn find_pub_key(req: &mut Request) -> Result<PublicKey, FromHexError> {
            let ref pub_key = req.extensions
                .get::<Router>()
                .unwrap()
                .find("pub_key")
                .unwrap();
            PublicKey::from_hex(pub_key)
        }

        fn bad_request(e: FromHexError, msg: &str) -> IronError {
            IronError::new(e, (Status::BadRequest, Header(ContentType::json()), msg))
        }

        /// Информация о пользователе
        fn get_user(&self, req: &mut Request) -> IronResult<Response> {

            let public_key = CryptoOwlsApi::find_pub_key(req).map_err(|e| {
                CryptoOwlsApi::bad_request(e, "\"Invalid request param: `pub_key`\"")
            })?;

            let user = {
                let snapshot = self.blockchain.snapshot();
                let schema = schema::CryptoOwlsSchema::new(snapshot);
                schema.users_proof().get(&public_key)
            };

            if let Some(user) = user {
                self.ok_response(&serde_json::to_value(user).unwrap())
            } else {
                self.not_found_response(&serde_json::to_value("User not found").unwrap())
            }
        }

        /// Полный список пользователей
        fn get_users(&self, _: &mut Request) -> IronResult<Response> {
            let snapshot = self.blockchain.snapshot();
            let schema = schema::CryptoOwlsSchema::new(snapshot);
            let idx = schema.users_proof();
            let users: Vec<User> = idx.values().collect();

            self.ok_response(&serde_json::to_value(&users).unwrap())

        }

        /// Информация о cове
        fn get_owl(&self, req: &mut Request) -> IronResult<Response> {
            let owl_hash = CryptoOwlsApi::find_owl_hash(req).map_err(|e| {
                CryptoOwlsApi::bad_request(e, "\"Invalid request param: `owl_hash`\"")
            })?;

            let owl = {
                let snapshot = self.blockchain.snapshot();
                let schema = schema::CryptoOwlsSchema::new(snapshot);
                schema.owls_state_proof().get(&owl_hash)
            };

            if let Some(owl) = owl {
                self.ok_response(&serde_json::to_value(owl).unwrap())
            } else {
                self.not_found_response(&serde_json::to_value("Owl not found").unwrap())
            }
        }

        /// Полный список сов
        fn get_owls(&self, _: &mut Request) -> IronResult<Response> {
            let snapshot = self.blockchain.snapshot();
            let schema = schema::CryptoOwlsSchema::new(snapshot);
            let idx = schema.owls_state_proof();
            let owls: Vec<CryptoOwlState> = idx.values().collect();
            self.ok_response(&serde_json::to_value(&owls).unwrap())
        }


        /// Cписок сов для пользователя
        fn get_users_owls(&self, req: &mut Request) -> IronResult<Response> {
            let users_key = CryptoOwlsApi::find_pub_key(req).map_err(|e| {
                CryptoOwlsApi::bad_request(e, "\"Invalid request param: `pub_key`\"")
            })?;

            let snapshot = self.blockchain.snapshot();
            let schema = schema::CryptoOwlsSchema::new(snapshot);

            if let Some(_) = schema.users_proof().get(&users_key) {
                let idx = schema.users_owls_view(&users_key);

                // type of iterator is ValueSetIndexIter<'_, Hash> !!!
                let owls: Vec<CryptoOwlState> = idx.iter()
                    .map(|h| schema.owls_state_proof().get(&h.1))
                    .collect::<Option<Vec<CryptoOwlState>>>()
                    .unwrap();
                self.ok_response(&serde_json::to_value(&owls).unwrap())
            } else {
                self.not_found_response(&serde_json::to_value("User not found").unwrap())
            }
        }


        /// Информация об ордерах на cову
        fn get_owls_orders(&self, req: &mut Request) -> IronResult<Response> {
            let owl_hash = CryptoOwlsApi::find_owl_hash(req).map_err(|e| {
                CryptoOwlsApi::bad_request(e, "\"Invalid request param: `owl_hash`\"")
            })?;
            let snapshot = self.blockchain.snapshot();
            let schema = schema::CryptoOwlsSchema::new(snapshot);

            if let Some(_) = schema.owls_state_proof().get(&owl_hash) {
                let idx = schema.owl_orders_view(&owl_hash);
                let orders: Vec<Order> = idx.iter()
                    .map(|h| schema.orders_proof_view().get(&h))
                    .collect::<Option<Vec<Order>>>()
                    .unwrap();
                self.ok_response(&serde_json::to_value(&orders).unwrap())
            } else {
                self.not_found_response(&serde_json::to_value("Owl not found").unwrap())
            }
        }

        /// Информация об ордерах выставленных юзером
        fn get_users_orders(&self, req: &mut Request) -> IronResult<Response> {
            let users_key = CryptoOwlsApi::find_pub_key(req).map_err(|e| {
                CryptoOwlsApi::bad_request(e, "\"Invalid request param: `pub_key`\"")
            })?;

            let snapshot = self.blockchain.snapshot();
            let schema = schema::CryptoOwlsSchema::new(snapshot);

            if let Some(_) = schema.users_proof().get(&users_key) {
                let idx = schema.user_orders_view(&users_key);

                let orders: Vec<Order> = idx.iter()
                    .map(|h| schema.orders_proof_view().get(&h))
                    .collect::<Option<Vec<Order>>>()
                    .unwrap();
                self.ok_response(&serde_json::to_value(&orders).unwrap())
            } else {
                self.not_found_response(&serde_json::to_value("User not found").unwrap())
            }
        }
    }
}

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, BytesN, Env, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug)]
pub struct DocumentRecord {
    pub doc_hash: BytesN<32>,
    pub entity_id: Symbol,
    pub entity_type: u32,
    pub doc_type: Symbol,
    pub version: u32,
    pub prior_hash: BytesN<32>,
    pub anchored_by: Address,
    pub ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Version(Symbol, Symbol),
    Document(Symbol, Symbol, u32),
    HashLookup(BytesN<32>),
    EntityTypes(Symbol),
}

#[contract]
pub struct RicardianRegistry;

#[contractimpl]
impl RicardianRegistry {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn anchor_document(
        env: Env,
        entity_id: Symbol,
        entity_type: u32,
        doc_type: Symbol,
        doc_hash: BytesN<32>,
    ) -> u32 {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if entity_type > 2 {
            panic!("invalid entity_type: must be 0, 1, or 2");
        }

        let hash_key = DataKey::HashLookup(doc_hash.clone());
        if env.storage().persistent().has(&hash_key) {
            panic!("document hash already anchored");
        }

        let ver_key = DataKey::Version(entity_id.clone(), doc_type.clone());
        let current_version: u32 = env
            .storage()
            .persistent()
            .get(&ver_key)
            .unwrap_or(0);

        let new_version = current_version + 1;

        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        let prior_hash = if current_version > 0 {
            let prior_key = DataKey::Document(
                entity_id.clone(),
                doc_type.clone(),
                current_version,
            );
            let prior_record: DocumentRecord =
                env.storage().persistent().get(&prior_key).unwrap();
            prior_record.doc_hash
        } else {
            zero_hash
        };

        let record = DocumentRecord {
            doc_hash: doc_hash.clone(),
            entity_id: entity_id.clone(),
            entity_type,
            doc_type: doc_type.clone(),
            version: new_version,
            prior_hash,
            anchored_by: admin.clone(),
            ledger: env.ledger().sequence(),
        };

        let doc_key = DataKey::Document(
            entity_id.clone(),
            doc_type.clone(),
            new_version,
        );
        env.storage().persistent().set(&doc_key, &record);

        env.storage().persistent().set(&hash_key, &(
            entity_id.clone(),
            doc_type.clone(),
            new_version,
        ));

        env.storage().persistent().set(&ver_key, &new_version);

        let types_key = DataKey::EntityTypes(entity_id.clone());
        let mut types: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&types_key)
            .unwrap_or(Vec::new(&env));

        let mut found = false;
        for i in 0..types.len() {
            if types.get(i).unwrap() == doc_type {
                found = true;
                break;
            }
        }
        if !found {
            types.push_back(doc_type.clone());
            env.storage().persistent().set(&types_key, &types);
        }

        env.events().publish(
            (symbol_short!("doc_anch"), entity_id.clone()),
            (doc_hash, doc_type, new_version),
        );

        new_version
    }

    pub fn get_document(
        env: Env,
        entity_id: Symbol,
        doc_type: Symbol,
        version: u32,
    ) -> DocumentRecord {
        let ver = if version == 0 {
            let ver_key = DataKey::Version(entity_id.clone(), doc_type.clone());
            env.storage()
                .persistent()
                .get(&ver_key)
                .unwrap_or_else(|| panic!("no documents found"))
        } else {
            version
        };

        let doc_key = DataKey::Document(entity_id, doc_type, ver);
        env.storage()
            .persistent()
            .get(&doc_key)
            .unwrap_or_else(|| panic!("document version not found"))
    }

    pub fn get_history(
        env: Env,
        entity_id: Symbol,
        doc_type: Symbol,
    ) -> Vec<DocumentRecord> {
        let ver_key = DataKey::Version(entity_id.clone(), doc_type.clone());
        let total: u32 = env
            .storage()
            .persistent()
            .get(&ver_key)
            .unwrap_or(0);

        let mut history = Vec::new(&env);
        for v in 1..=total {
            let doc_key = DataKey::Document(
                entity_id.clone(),
                doc_type.clone(),
                v,
            );
            if let Some(record) = env.storage().persistent().get(&doc_key) {
                history.push_back(record);
            }
        }
        history
    }

    pub fn get_all_documents(
        env: Env,
        entity_id: Symbol,
    ) -> Vec<DocumentRecord> {
        let types_key = DataKey::EntityTypes(entity_id.clone());
        let types: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&types_key)
            .unwrap_or(Vec::new(&env));

        let mut docs = Vec::new(&env);
        for i in 0..types.len() {
            let dt = types.get(i).unwrap();
            let ver_key = DataKey::Version(entity_id.clone(), dt.clone());
            if let Some(ver) = env.storage().persistent().get::<_, u32>(&ver_key) {
                let doc_key = DataKey::Document(
                    entity_id.clone(),
                    dt,
                    ver,
                );
                if let Some(record) = env.storage().persistent().get(&doc_key) {
                    docs.push_back(record);
                }
            }
        }
        docs
    }

    pub fn verify_document(
        env: Env,
        doc_hash: BytesN<32>,
    ) -> Option<DocumentRecord> {
        let hash_key = DataKey::HashLookup(doc_hash);
        if let Some((entity_id, doc_type, version)) = env
            .storage()
            .persistent()
            .get::<_, (Symbol, Symbol, u32)>(&hash_key)
        {
            let doc_key = DataKey::Document(entity_id, doc_type, version);
            env.storage().persistent().get(&doc_key)
        } else {
            None
        }
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }
}

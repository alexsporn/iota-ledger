use std::collections::HashSet;

use iota_sdk::{
    IotaClient,
    rpc_types::{IotaObjectData, IotaObjectDataOptions, IotaObjectResponse},
    types::{
        base_types::{ObjectID, ObjectType},
        object::{MoveObject, Object},
        transaction::{InputObjectKind, TransactionData, TransactionDataAPI},
    },
};

pub(crate) async fn load_objects_with_client(
    client: &IotaClient,
    transaction: &TransactionData,
) -> Result<Vec<Object>, anyhow::Error> {
    let object_ids = object_ids_from_transaction(transaction)?;

    if object_ids.is_empty() {
        return Ok(vec![]);
    }

    let responses = client
        .read_api()
        .multi_get_object_with_options(object_ids, IotaObjectDataOptions::bcs_lossless())
        .await?;

    let objects: Vec<Object> = responses
        .iter()
        .map(|resp| object_from(resp.clone()))
        .collect();

    Ok(objects)
}

fn object_ids_from_transaction(
    transaction: &TransactionData,
) -> Result<Vec<ObjectID>, anyhow::Error> {
    let object_ids = transaction
        .gas_data()
        .payment
        .iter()
        .map(|payment| payment.0);

    let input_objects = transaction
        .input_objects()?
        .into_iter()
        .filter_map(|input| match input {
            InputObjectKind::ImmOrOwnedMoveObject(id) => Some(id.0),
            _ => None,
        });

    let mut unique_ids = HashSet::new();
    unique_ids.extend(object_ids);
    unique_ids.extend(input_objects);

    Ok(unique_ids.into_iter().collect())
}

fn object_from(resp: IotaObjectResponse) -> Object {
    let data: IotaObjectData = resp.data.unwrap();

    let t = if let ObjectType::Struct(t) = data.type_.unwrap() {
        t
    } else {
        panic!("unexpected enum variant");
    };

    let bcs_bytes = match data.bcs.unwrap() {
        iota_sdk::rpc_types::IotaRawData::MoveObject(move_obj) => move_obj.bcs_bytes,
        _ => panic!("Expected move object"),
    };

    let o = Object::new_move(
        MoveObject::new_from_execution_with_limit(t, data.version, bcs_bytes, 250 * 1024).unwrap(),
        data.owner.unwrap(),
        data.previous_transaction.unwrap(),
    );

    let mut inner = o.into_inner();
    inner.storage_rebate = data.storage_rebate.unwrap_or(0);

    inner.into()
}

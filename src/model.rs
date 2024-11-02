use crate::model::MeasurementError::{MissingValue, UnexpectedValueType};
use sml_rs::parser::common::Value;
use sml_rs::parser::complete::{File, MessageBody};
use sml_rs::parser::OctetStr;
use thiserror::Error;

#[cfg(not(feature = "producer_consumer_swapped"))]
const OBIS_CONSUMED: OctetStr<'_> = &[1, 0, 1, 8, 0, 255];
#[cfg(feature = "producer_consumer_swapped")]
const OBIS_PRODUCED: OctetStr<'_> = &[1, 0, 1, 8, 0, 255];

#[cfg(not(feature = "producer_consumer_swapped"))]
const OBIS_PRODUCED: OctetStr<'_> = &[1, 0, 2, 8, 0, 255];

#[cfg(feature = "producer_consumer_swapped")]
const OBIS_CONSUMED: OctetStr<'_> = &[1, 0, 2, 8, 0, 255];

#[derive(Debug, PartialEq, Default)]
pub struct Measurement {
    pub consumed: u64,
    pub produced: u64,
}
#[derive(Error, Debug)]
pub enum MeasurementError {
    #[error("required message `GetListResponse` not found")]
    ListResponseNotFound,
    #[error("got a ListEntry with an unexpected value type for OBIS-type `{0:?}`")]
    UnexpectedValueType(OctetStr<'static>),
    #[error("required value for OBIS-type `{0:?}` not found")]
    MissingValue(OctetStr<'static>),
}
impl TryFrom<File<'_>> for Measurement {
    type Error = MeasurementError;

    fn try_from(value: File<'_>) -> Result<Self, Self::Error> {
        let list_response = value
            .messages
            .iter()
            .find_map(|msg| match &msg.message_body {
                MessageBody::GetListResponse(r) => Some(&r.val_list),
                _ => None,
            })
            .ok_or(MeasurementError::ListResponseNotFound)?;

        let result = list_response.iter().try_fold::<_, _, _>(
            (None, None),
            |(consumed, produced), cur| match (cur.obj_name, &cur.value) {
                (OBIS_CONSUMED, Value::U64(consumed)) => Ok((Some(*consumed), produced)),
                (OBIS_CONSUMED, _) => Err(UnexpectedValueType(OBIS_CONSUMED)),
                (OBIS_PRODUCED, Value::U64(produced)) => Ok((consumed, Some(*produced))),
                (OBIS_PRODUCED, _) => Err(UnexpectedValueType(OBIS_CONSUMED)),
                _ => Ok((consumed, produced)),
            },
        )?;

        match result {
            (Some(consumed), Some(produced)) => Ok(Measurement { consumed, produced }),
            (Some(_), _) => Err(MissingValue(OBIS_PRODUCED)),
            _ => Err(MissingValue(OBIS_CONSUMED)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::model::Measurement;
    use sml_rs::parser::common::Status::Status32;
    use sml_rs::parser::common::Time::SecIndex;
    use sml_rs::parser::common::{CloseResponse, ListEntry, OpenResponse, Value};
    use sml_rs::parser::complete::{File, GetListResponse, Message, MessageBody};

    #[test]
    fn test() {
        // see https://de.wikipedia.org/wiki/OBIS-Kennzahlen#Elektrische_Energie
        let sample = File {
            messages: vec![
                Message {
                    transaction_id: &[4, 16, 103, 130],
                    group_no: 0,
                    abort_on_error: 0,
                    message_body: MessageBody::OpenResponse(OpenResponse {
                        codepage: None,
                        client_id: Some(&[255, 255, 255, 255, 255, 255]),
                        req_file_id: &[1, 90, 205, 43],
                        server_id: &[10, 1, 76, 71, 90, 0, 3, 153, 70, 27],
                        ref_time: Some(SecIndex(77179560)),
                        sml_version: None,
                    }),
                },
                Message {
                    transaction_id: &[4, 16, 103, 131],
                    group_no: 0,
                    abort_on_error: 0,
                    message_body: MessageBody::GetListResponse(GetListResponse {
                        client_id: Some(&[255, 255, 255, 255, 255, 255]),
                        server_id: &[10, 1, 76, 71, 90, 0, 3, 153, 70, 27],
                        list_name: Some(&[1, 0, 98, 10, 255, 255]),
                        act_sensor_time: Some(SecIndex(77179560)),
                        val_list: vec![
                            ListEntry {
                                // 1-0:96.50.1
                                obj_name: &[1, 0, 96, 50, 1, 1],
                                status: None,
                                val_time: None,
                                unit: None,
                                scaler: None,
                                value: Value::Bytes(&[76, 71, 90]),
                                value_signature: None,
                            },
                            ListEntry {
                                // 1-0:96.1.0
                                obj_name: &[1, 0, 96, 1, 0, 255],
                                status: None,
                                val_time: None,
                                unit: None,
                                scaler: None,
                                value: Value::Bytes(&[10, 1, 76, 71, 90, 0, 3, 153, 70, 27]),
                                value_signature: None,
                            },
                            ListEntry {
                                // 1-0:1.8.0
                                obj_name: &[1, 0, 1, 8, 0, 255],
                                status: Some(Status32(1878276u32)),
                                val_time: Some(SecIndex(77179560)),
                                unit: Some(30),
                                scaler: Some(3),
                                value: Value::U64(23152u64),
                                value_signature: None,
                            },
                            ListEntry {
                                // 1-2:2.8.0
                                obj_name: &[1, 0, 2, 8, 0, 255],
                                status: None,
                                val_time: Some(SecIndex(77179560)),
                                unit: Some(30),
                                scaler: Some(3),
                                value: Value::U64(8564u64),
                                value_signature: None,
                            },
                        ],
                        list_signature: None,
                        act_gateway_time: None,
                    }),
                },
                Message {
                    transaction_id: &[4, 16, 103, 132],
                    group_no: 0,
                    abort_on_error: 0,
                    message_body: MessageBody::CloseResponse(CloseResponse {
                        global_signature: None,
                    }),
                },
            ],
        };

        let expected = Measurement {
            consumed: 8564,
            produced: 23152,
        };

        let actual = sample.try_into().unwrap();
        assert_eq!(expected, actual);
    }
}

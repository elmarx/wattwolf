use crate::config::Sensor;
use crate::error::MeasurementError;

#[derive(Debug, PartialEq)]
pub struct Measurement<'a> {
    sensor: &'a Sensor,
    pub value: u64,
}

pub fn read_measurement<'a>(
    file: &sml_rs::parser::complete::File,
    sensors: &'a [Sensor],
) -> Result<Vec<Measurement<'a>>, MeasurementError> {
    let list_response = file
        .messages
        .iter()
        .find_map(|msg| {
            if let sml_rs::parser::complete::MessageBody::GetListResponse(resp) = &msg.message_body
            {
                Some(resp)
            } else {
                None
            }
        })
        .ok_or(MeasurementError::ListResponseNotFound)?;

    let mut measurements = Vec::new();

    for sensor in sensors {
        let entry = list_response
            .val_list
            .iter()
            .find(|entry| entry.obj_name == sensor.obis.exact)
            .ok_or(MeasurementError::MissingValue(sensor.obis.exact))?;

        match &entry.value {
            sml_rs::parser::common::Value::U64(val) => measurements.push(Measurement {
                sensor,
                value: *val,
            }),
            _ => return Err(MeasurementError::UnexpectedValueType(sensor.obis.exact)),
        }
    }

    Ok(measurements)
}

#[cfg(test)]
mod test {
    use crate::config::Sensor;
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

        let sample_sensors: Vec<Sensor> = vec![
            Sensor {
                name: "consumed".to_string(),
                friendly_name: "Energy Consumed".to_string(),
                obis: &crate::obis::OBIS_1_8_0,
                device_class: None,
                state_class: None,
            },
            Sensor {
                name: "produced".to_string(),
                friendly_name: "Energy Produced".to_string(),
                obis: &crate::obis::OBIS_2_8_0,
                device_class: None,
                state_class: None,
            },
        ];

        let expected = vec![
            Measurement {
                sensor: &sample_sensors[0],
                value: 23152,
            },
            Measurement {
                sensor: &sample_sensors[1],
                value: 8564,
            },
        ];

        let actual = crate::model::read_measurement(&sample, &sample_sensors).unwrap();
        assert_eq!(expected, actual);
    }
}

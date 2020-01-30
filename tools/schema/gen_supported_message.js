var _ = require("lodash");
var fs = require("fs");
var xml2js = require("xml2js");

var settings = require("./settings");
let util = require("./util");

function generate_supported_message(message_types) {
    var file_name = "supported_message.rs";
    var file_path = `${settings.rs_supported_message_dir}/${file_name}`;

    var contents = `// This file was autogenerated by tools/schema/gen_supported_message.js
// DO NOT EDIT THIS FILE

use std::io::{Read, Write};

use opcua_types::{
    encoding::*,
    node_id::NodeId,
    node_ids::ObjectId,
    service_types::*,
};

pub use crate::comms::tcp_types::AcknowledgeMessage;

/// This macro helps avoid tedious repetition as new messages are added
/// The first form just handles the trailing comma after the last entry to save some pointless
/// editing when new messages are added to the list.
macro_rules! supported_messages_enum {
    [ $( $x:ident, ) * ] => (supported_messages_enum![ $( $x ),* ];);
    [ $( $x:ident ), * ] => {
        #[derive(Debug, PartialEq, Clone)]
        pub enum SupportedMessage {
            /// An invalid request / response of some form
            Invalid(ObjectId),
            /// Acknowledge message
            AcknowledgeMessage(Box<AcknowledgeMessage>),
            /// Other messages
            $( $x(Box<$x>), )*
        }

        impl BinaryEncoder <SupportedMessage> for SupportedMessage {
            fn byte_len(&self) -> usize {
                match self {
                    SupportedMessage::Invalid(object_id) => {
                        panic!("Unsupported message byte_len {:?}", object_id);
                    },
                    SupportedMessage::AcknowledgeMessage(value) => value.byte_len(),
                    $( SupportedMessage::$x(value) => value.byte_len(), )*
                }
            }

            fn encode<S: Write>(&self, stream: &mut S) -> EncodingResult<usize> {
                match self {
                    SupportedMessage::Invalid(object_id) => {
                        panic!("Unsupported message encode {:?}", object_id);
                    },
                    SupportedMessage::AcknowledgeMessage(value) => value.encode(stream),
                    $( SupportedMessage::$x(value) => value.encode(stream), )*
                }
            }

            fn decode<S: Read>(_: &mut S, _: &DecodingLimits) -> EncodingResult<Self> {
                // THIS WILL NOT DO ANYTHING
                panic!("Cannot decode a stream to a supported message type");
            }
        }

        impl Into<SupportedMessage> for AcknowledgeMessage{
            fn into(self) -> SupportedMessage { SupportedMessage::AcknowledgeMessage(Box::new(self)) }
        }

        $(
        impl Into<SupportedMessage> for $x {
            fn into(self) -> SupportedMessage { SupportedMessage::$x(Box::new(self)) }
        }
        )*

        impl SupportedMessage {
            pub fn node_id(&self) -> NodeId {
                match self {
                    SupportedMessage::Invalid(object_id) => {
                        panic!("Unsupported message invalid, node_id {:?}", object_id);
                    },
                    SupportedMessage::AcknowledgeMessage(value) => {
                        panic!("Unsupported message node_id {:?}", value);
                    },
                    $( SupportedMessage::$x(value) => value.object_id().into(), )*
                }
            }
        }
    }
}

impl SupportedMessage {
    pub fn request_handle(&self) -> u32 {
        match self {
            SupportedMessage::Invalid(_) | SupportedMessage::AcknowledgeMessage(_) => 0,
`;

    _.each(message_types, message_type => {
        if (message_type.endsWith("Request")) {
            contents += `            SupportedMessage::${message_type}(r) => r.request_header.request_handle,
`;
        } else if (message_type.endsWith("Response") || message_type === "ServiceFault") {
            contents += `            SupportedMessage::${message_type}(r) => r.response_header.request_handle,
`;
        }
    });

    contents += `        }
    }

    pub fn decode_by_object_id<S: Read>(stream: &mut S, object_id: ObjectId, decoding_limits: &DecodingLimits) -> EncodingResult<Self> {
        trace!("decoding object_id {:?}", object_id);
        let decoded_message = match object_id {
`;

    _.each(message_types, message_type => {
        contents += `            ObjectId::${message_type}_Encoding_DefaultBinary => {
                ${message_type}::decode(stream, decoding_limits)?.into()
            }
`;
    });

    contents += `            _ => {
                debug!("decoding unsupported for object id {:?}", object_id);
                SupportedMessage::Invalid(object_id)
            }
        };
        Ok(decoded_message)
    }
}

// These are all the messages handled into and out of streams by the OPCUA server / client code
supported_messages_enum![
`;

    _.each(message_types, message_type => {
        contents += `    ${message_type},
`;
    });

    contents += `];
`;

    util.write_to_file(file_path, contents);
}


// These types are messages which means they implement Into<SupportedMessage> and are processed by supported message
generate_supported_message([
    // A service fault, returned when the service failed
    "ServiceFault",
    // Secure channel service
    "OpenSecureChannelRequest", "OpenSecureChannelResponse",
    "CloseSecureChannelRequest", "CloseSecureChannelResponse",
    // Discovery service
    "GetEndpointsRequest", "GetEndpointsResponse",
    "FindServersRequest", "FindServersResponse",
    "RegisterServerRequest", "RegisterServerResponse",
    // Session service
    "CreateSessionRequest", "CreateSessionResponse",
    "CloseSessionRequest", "CloseSessionResponse",
    "CancelRequest", "CancelResponse",
    "ActivateSessionRequest", "ActivateSessionResponse",
    // Node management service
    "AddNodesRequest", "AddNodesResponse",
    "AddReferencesRequest", "AddReferencesResponse",
    "DeleteNodesRequest", "DeleteNodesResponse",
    "DeleteReferencesRequest", "DeleteReferencesResponse",
    // MonitoredItem service
    "CreateMonitoredItemsRequest", "CreateMonitoredItemsResponse",
    "ModifyMonitoredItemsRequest", "ModifyMonitoredItemsResponse",
    "DeleteMonitoredItemsRequest", "DeleteMonitoredItemsResponse",
    "SetMonitoringModeRequest", "SetMonitoringModeResponse",
    "SetTriggeringRequest", "SetTriggeringResponse",
    // Subscription service
    "CreateSubscriptionRequest", "CreateSubscriptionResponse",
    "ModifySubscriptionRequest", "ModifySubscriptionResponse",
    "DeleteSubscriptionsRequest", "DeleteSubscriptionsResponse",
    "TransferSubscriptionsRequest", "TransferSubscriptionsResponse",
    "SetPublishingModeRequest", "SetPublishingModeResponse",
    // View service
    "BrowseRequest", "BrowseResponse",
    "BrowseNextRequest", "BrowseNextResponse",
    "PublishRequest", "PublishResponse",
    "RepublishRequest", "RepublishResponse",
    "TranslateBrowsePathsToNodeIdsRequest", "TranslateBrowsePathsToNodeIdsResponse",
    "RegisterNodesRequest", "RegisterNodesResponse",
    "UnregisterNodesRequest", "UnregisterNodesResponse",
    // Attribute service
    "ReadRequest", "ReadResponse",
    "HistoryReadRequest", "HistoryReadResponse",
    "WriteRequest", "WriteResponse",
    "HistoryUpdateRequest", "HistoryUpdateResponse",
    // Method service
    "CallRequest", "CallResponse",
]);
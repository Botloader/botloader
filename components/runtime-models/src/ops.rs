use crate::{
    discord::{guild::Guild, role::Role},
    internal::{
        channel::{
            CreateForumThread, CreateThread, CreateThreadFromMessage, EditGuildChannelPosition,
            ForumThreadResponse, GuildChannel, ListThreadMembersRequest, ListThreadsRequest,
            ThreadMember, ThreadsListing, UpdateThread,
        },
        messages::{
            Message, OpCreateChannelMessage, OpDeleteMessage, OpDeleteMessagesBulk,
            OpEditChannelMessage, OpGetMessages,
        },
        role::{OpCreateRoleFields, OpUpdateRoleFields},
    },
};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// macro_rules! ops_sync {
//     ($($op_name:ident($arg:ty) => $returnType:ty),+) => {
//         #[allow(non_camel_case_types)]
//         #[derive(Clone, Debug, Deserialize, TS)]
//         #[ts(export, rename = "EasyOpsSync", export_to = "bindings/internal/EasyOpsSync.ts")]
//         #[serde(tag = "kind", content = "arg")]
//         #[serde(rename_all = "snake_case")]
//         pub enum EasyOpsSync{
//             $(
//                 $op_name($arg),
//             )*
//         }

//         #[allow(non_camel_case_types)]
//         #[derive(Clone, Debug, Serialize, TS)]
//         #[ts(export, rename = "EasyOpsReturnTypesSync", export_to = "bindings/internal/EasyOpsReturnTypesSync.ts")]
//         struct EasyOpsReturnTypesSync {
//             $(
//                 $op_name: $returnType,
//             )*
//         }

//         pub trait EasyOpsHandlerSync{
//             $(
//                 fn $op_name(&self, arg: $arg) -> Result<$returnType, anyhow::Error>;
//             )*
//         }

//         fn handle_op<T: EasyOpsHandlerSync>(handler: &T, op: EasyOpsSync) -> Result<serde_json::Value, anyhow::Error>{
//              match op{
//                 $(
//                     EasyOpsSync::$op_name(arg) => handler.$op_name(arg).map(|v| serde_json::to_value(v).unwrap()),
//                 )*
//             }
//         }
//     };
// }
#[macro_export]
macro_rules! ops_async {
    ($($op_name:ident($arg:ty) => $returnType:ty),+) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Debug, Deserialize, TS)]
        #[ts(export, rename = "EasyOpsASync", export_to = "bindings/internal/EasyOpsASync.ts")]
        #[serde(tag = "kind", content = "arg")]
        #[serde(rename_all = "snake_case")]
        pub enum EasyOpsASync{
            $(
                $op_name($arg),
            )*
        }

        #[allow(non_camel_case_types)]
        #[derive(Clone, Debug, Serialize, TS)]
        #[ts(export, rename = "EasyOpsReturnTypesASync", export_to = "bindings/internal/EasyOpsReturnTypesASync.ts")]
        struct EasyOpsReturnTypesASync {
            $(
                $op_name: $returnType,
            )*
        }

        pub trait EasyOpsHandlerASync{
            $(
                #[allow(async_fn_in_trait)]
                async fn $op_name(&self, arg: $arg) -> Result<$returnType, anyhow::Error>;
            )*
        }

        pub async fn handle_async_op<T: EasyOpsHandlerASync>(handler: &T, op: EasyOpsASync) -> Result<serde_json::Value, anyhow::Error>{
             match op{
                $(
                    EasyOpsASync::$op_name(arg) => handler.$op_name(arg).await.map(|v| serde_json::to_value(v).unwrap()),
                )*
            }
        }
    };
}

ops_async! {
    discord_get_guild(()) => Guild,

    // channel_id, message_id
    discord_get_message((String, String)) => Message,
    discord_get_messages(OpGetMessages) => Vec<Message>,
    discord_create_message(OpCreateChannelMessage) => Message,
    discord_edit_message(OpEditChannelMessage) => Message,
    // channel_id, message_id
    discord_crosspost_message((String,String)) => (),
    discord_delete_message(OpDeleteMessage) => (),
    discord_bulk_delete_messages(OpDeleteMessagesBulk) => (),

    discord_start_thread_from_message(CreateThreadFromMessage) => GuildChannel,
    discord_start_thread_without_message(CreateThread) => GuildChannel,
    discord_start_forum_thread(CreateForumThread) => ForumThreadResponse,
    // channel_id, user_id
    discord_add_thread_member((String, String)) => (),
    discord_remove_thread_member((String, String)) => (),
    discord_list_thread_members(ListThreadMembersRequest) => Vec<ThreadMember>,
    discord_list_active_threads(()) => ThreadsListing,
    discord_list_public_archived_threads(ListThreadsRequest) => ThreadsListing,
    discord_list_private_archived_threads(ListThreadsRequest) => ThreadsListing,
    discord_edit_thread(UpdateThread) => GuildChannel,

    discord_bulk_edit_channels(Vec<EditGuildChannelPosition>) => (),

    discord_create_role(OpCreateRoleFields) => Role,
    discord_update_role(OpUpdateRoleFields) => Role,
    discord_delete_role(String) => ()
}

use crate::
{
	discord::{
		component::Component,
		embed::Embed,
		message::{
			Attachment, MessageFlags, MessageType,
		},
	},
	internal::{
		messages::UserMention,
	},
	util::NotBigU64
};

use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(
	export_to = "bindings/internal/IMessageSnapshot.ts",
    rename = "IMessageSnapshot"
)]
pub struct MessageSnapshot {
	pub message: MessageSnapshotFields,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[ts(optional)]
	pub guild_id: Option<String>,
}

impl TryFrom<twilight_model::channel::message::MessageSnapshot> for MessageSnapshot {
	type Error = anyhow::Error;

	fn try_from(v: twilight_model::channel::message::MessageSnapshot) -> Result<Self, Self::Error> {
		Ok(Self {
			message: MessageSnapshotFields::try_from(v.clone())?,
			guild_id: v.guild_id.map(|id| id.to_string()),
		})
	}
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(
	export_to = "bindings/internal/IMessageSnapshotFields.ts",
    rename = "IMessageSnapshotFields"
)]
pub struct MessageSnapshotFields {
	pub attachments: Vec<Attachment>,
	
	pub components: Vec<Component>,

	pub content: String,
	pub edited_timestamp: Option<NotBigU64>,

	pub embeds: Vec<Embed>,
	
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
	pub flags: Option<MessageFlags>,

	pub kind: MessageType,
	
	pub mentions: Vec<UserMention>,
	pub mention_roles: Vec<String>,
	pub timestamp: NotBigU64,
}

impl TryFrom<twilight_model::channel::message::MessageSnapshot> for MessageSnapshotFields {
	type Error = anyhow::Error;

	fn try_from(v: twilight_model::channel::message::MessageSnapshot) -> Result<Self, Self::Error> {
		let m = v.message;
		Ok(Self {
			attachments: m.attachments.into_iter().map(From::from).collect(),
			components: m
				.components
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<_, _>>()?,
			content: m.content,
			edited_timestamp: m
                .edited_timestamp
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
			embeds: m.embeds.into_iter().map(From::from).collect(),
			flags: m.flags.map(From::from),
			kind: m.kind.try_into()?,
			mentions: m.mentions.into_iter().map(From::from).collect(),
			mention_roles: m.mention_roles.iter().map(ToString::to_string).collect(),
			timestamp: NotBigU64(m.timestamp.as_micros() as u64 / 1000),
		})
	}
}
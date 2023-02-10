use crate::config::{
    ConfigStore, ConfigStoreError, ConfigStoreResult, CreatePlugin, CreateScript,
    CreateUpdatePremiumSlotBySource, GuildMetaConfig, JoinedGuild, PremiumSlot, Script,
    ScriptContributes, UpdatePluginMeta, UpdateScript,
};
use async_trait::async_trait;
use common::{plugin::Plugin, user::UserMeta};
use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

#[derive(Debug, Clone)]
pub struct ReadOnlyConfigStore {
    guild_id: Id<GuildMarker>,
    scripts: Vec<Script>,
}

#[async_trait]
impl ConfigStore for ReadOnlyConfigStore {
    async fn get_script(
        &self,
        guild_id: Id<GuildMarker>,
        script_name: String,
    ) -> ConfigStoreResult<Script> {
        if guild_id != self.guild_id {
            return Err(ConfigStoreError::ScriptNotFound);
        }

        self.scripts
            .iter()
            .find(|item| item.name == script_name)
            .map(Clone::clone)
            .ok_or(ConfigStoreError::ScriptNotFound)
    }

    async fn get_script_by_id(
        &self,
        guild_id: Id<GuildMarker>,
        script_id: u64,
    ) -> ConfigStoreResult<Script> {
        if guild_id != self.guild_id {
            return Err(ConfigStoreError::ScriptNotFound);
        }

        self.scripts
            .iter()
            .find(|item| item.id == script_id)
            .map(Clone::clone)
            .ok_or(ConfigStoreError::ScriptNotFound)
    }

    async fn create_script(
        &self,
        _guild_id: Id<GuildMarker>,
        _script: CreateScript,
    ) -> ConfigStoreResult<Script> {
        todo!();
    }

    async fn update_script(
        &self,
        _guild_id: Id<GuildMarker>,
        _script: UpdateScript,
    ) -> ConfigStoreResult<Script> {
        todo!();
    }

    async fn update_script_contributes(
        &self,
        guild_id: Id<GuildMarker>,
        script_id: u64,
        contribs: ScriptContributes,
    ) -> ConfigStoreResult<Script> {
        let mut script = self.get_script_by_id(guild_id, script_id).await?;
        script.contributes = contribs;

        Ok(script)
    }

    async fn del_script(
        &self,
        _guild_id: Id<GuildMarker>,
        _script_name: String,
    ) -> ConfigStoreResult<()> {
        todo!();
    }

    async fn list_scripts(&self, guild_id: Id<GuildMarker>) -> ConfigStoreResult<Vec<Script>> {
        if guild_id != self.guild_id {
            return Ok(Vec::new());
        }

        return Ok(self.scripts.clone());
    }

    async fn get_guild_meta_config(
        &self,
        _guild_id: Id<GuildMarker>,
    ) -> ConfigStoreResult<Option<GuildMetaConfig>> {
        Ok(None)
    }

    async fn update_guild_meta_config(
        &self,
        _conf: &GuildMetaConfig,
    ) -> ConfigStoreResult<GuildMetaConfig> {
        todo!();
    }

    async fn add_update_joined_guild(&self, guild: JoinedGuild) -> ConfigStoreResult<JoinedGuild> {
        Ok(guild)
    }

    async fn get_joined_guilds(
        &self,
        _ids: &[Id<GuildMarker>],
    ) -> ConfigStoreResult<Vec<JoinedGuild>> {
        todo!();
    }

    async fn get_joined_guilds_not_in(
        &self,
        _ids: &[Id<GuildMarker>],
    ) -> ConfigStoreResult<Vec<JoinedGuild>> {
        todo!();
    }

    async fn set_guild_left_status(
        &self,
        _guild_id: Id<GuildMarker>,
        _left: bool,
    ) -> ConfigStoreResult<JoinedGuild> {
        todo!();
    }

    async fn is_guild_whitelisted(&self, _id: Id<GuildMarker>) -> ConfigStoreResult<bool> {
        Ok(true)
    }

    async fn delete_guild_config_data(&self, _id: Id<GuildMarker>) -> ConfigStoreResult<()> {
        Ok(())
    }

    async fn get_left_guilds(&self, _threshold_hours: u64) -> ConfigStoreResult<Vec<JoinedGuild>> {
        todo!();
    }

    async fn get_guild_premium_slots(
        &self,
        _guild_id: Id<GuildMarker>,
    ) -> ConfigStoreResult<Vec<PremiumSlot>> {
        todo!()
    }

    async fn get_user_premium_slots(
        &self,
        _user_id: Id<UserMarker>,
    ) -> ConfigStoreResult<Vec<PremiumSlot>> {
        todo!()
    }

    async fn create_update_premium_slot_by_source(
        &self,
        _slot: CreateUpdatePremiumSlotBySource,
    ) -> ConfigStoreResult<PremiumSlot> {
        todo!()
    }

    async fn update_premium_slot_attachment(
        &self,
        _user_id: Id<UserMarker>,
        _slot_id: u64,
        _guild_id: Option<Id<GuildMarker>>,
    ) -> ConfigStoreResult<PremiumSlot> {
        todo!()
    }

    async fn create_plugin(&self, _create_plugin: CreatePlugin) -> ConfigStoreResult<Plugin> {
        todo!()
    }
    async fn update_plugin_meta(
        &self,
        _plugin_id: u64,
        _update_plugin: UpdatePluginMeta,
    ) -> ConfigStoreResult<Plugin> {
        todo!()
    }
    async fn update_script_plugin_dev_version(
        &self,
        _plugin_id: u64,
        _new_source: String,
    ) -> ConfigStoreResult<Plugin> {
        todo!()
    }
    async fn publish_script_plugin_version(
        &self,
        _plugin_id: u64,
        _new_source: String,
    ) -> ConfigStoreResult<Plugin> {
        todo!()
    }

    async fn get_user_meta(&self, _user_id: u64) -> ConfigStoreResult<UserMeta> {
        todo!()
    }

    async fn get_plugin(&self, _plugin_id: u64) -> ConfigStoreResult<Plugin> {
        todo!()
    }

    async fn get_user_plugins(&self, _user_id: u64) -> ConfigStoreResult<Vec<Plugin>> {
        todo!()
    }

    async fn get_published_public_plugins(&self) -> ConfigStoreResult<Vec<Plugin>> {
        todo!()
    }

    async fn try_guild_add_script_plugin(
        &self,
        _guild_id: Id<GuildMarker>,
        _plugin_id: u64,
        _auto_update: bool,
    ) -> ConfigStoreResult<Script> {
        todo!()
    }
}

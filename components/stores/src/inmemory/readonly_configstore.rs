use crate::config::{
    ConfigStore, ConfigStoreError, ConfigStoreResult, CreateScript, GuildMetaConfig, JoinedGuild,
    Script, ScriptContributes, UpdateScript,
};
use async_trait::async_trait;
use twilight_model::id::GuildId;

#[derive(Debug, Clone)]
pub struct ReadOnlyConfigStore {
    guild_id: GuildId,
    scripts: Vec<Script>,
}

#[async_trait]
impl ConfigStore for ReadOnlyConfigStore {
    async fn get_script(
        &self,
        guild_id: GuildId,
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
        guild_id: GuildId,
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
        _guild_id: GuildId,
        _script: CreateScript,
    ) -> ConfigStoreResult<Script> {
        todo!();
    }

    async fn update_script(
        &self,
        _guild_id: GuildId,
        _script: UpdateScript,
    ) -> ConfigStoreResult<Script> {
        todo!();
    }

    async fn update_script_contributes(
        &self,
        guild_id: GuildId,
        script_id: u64,
        contribs: ScriptContributes,
    ) -> ConfigStoreResult<Script> {
        let mut script = self.get_script_by_id(guild_id, script_id).await?;
        script.contributes = contribs;

        Ok(script)
    }

    async fn del_script(&self, _guild_id: GuildId, _script_name: String) -> ConfigStoreResult<()> {
        todo!();
    }

    async fn list_scripts(&self, guild_id: GuildId) -> ConfigStoreResult<Vec<Script>> {
        if guild_id != self.guild_id {
            return Ok(Vec::new());
        }

        return Ok(self.scripts.clone());
    }

    async fn get_guild_meta_config(
        &self,
        _guild_id: GuildId,
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

    async fn get_joined_guilds(&self, _ids: &[GuildId]) -> ConfigStoreResult<Vec<JoinedGuild>> {
        todo!();
    }

    async fn set_guild_left_status(
        &self,
        guild_id: GuildId,
        left: bool,
    ) -> ConfigStoreResult<JoinedGuild> {
        todo!();
    }

    async fn is_guild_whitelisted(&self, _id: GuildId) -> ConfigStoreResult<bool> {
        Ok(true)
    }
}

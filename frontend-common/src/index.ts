export const BOTLOADER_TEST_VAR = 10;
export * from './api_client';
export * from './generated/api';
// explicit re-export wins over the generated (untyped) FullGuild from the star export above
export { FullGuild, Guild, DiscordChannel, DiscordRole, DiscordNumberedChannelTypes } from './discord_models';
import { type ExractClassProperties } from '../core_util';
import * as Internal from '../generated/internal/index';

export type CustomEmojiFields = ExractClassProperties<CustomEmoji>

/**
 * Represents a custom discord server emoji
 */
export class CustomEmoji {
    id: string;
    animated: boolean;
    available: boolean;
    managed: boolean;
    name: string;
    requireColons: boolean;

    /**
     * Roles that can use the emoji
     */
    roles: string[] | null;

    createdByUserId: string | null;

    constructor(fields: CustomEmojiFields) {
        this.id = fields.id
        this.animated = fields.animated
        this.available = fields.available
        this.managed = fields.managed
        this.name = fields.name
        this.requireColons = fields.requireColons
        this.roles = fields.roles
        this.createdByUserId = fields.createdByUserId
    }

    static fromInternal(data: Internal.CustomEmoji) {
        return new CustomEmoji({
            ...data,
            roles: data.roles.length === 0 ? null : data.roles
        })
    }

    static format(id: string, name?: string) {
        return `<:${name ?? "custom_emoji"}:id>`
    }

    format() {
        return CustomEmoji.format(this.id, this.name)
    }
}
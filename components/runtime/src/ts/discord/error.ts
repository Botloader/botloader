export class NotFoundError extends Error {
    message: string;

    constructor(message: string) {
        super(message)
        this.message = "Discord.NotFoundError: " + message;
    }
}

export class PermissionsError extends Error {
    message: string;

    constructor(message: string) {
        super(message)
        this.message = "Discord.PermissionsError: " + message;
    }
}

export class LimitReachedError extends Error {
    message: string;

    constructor(message: string) {
        super(message)
        this.message = "Discord.LimitReachedError: " + message;
    }
}

export class ServerError extends Error {
    message: string;

    constructor(message: string) {
        super(message)
        this.message = "Discord.ServerError: " + message;
    }
}

export class GenericError extends Error {
    message: string;

    constructor(message: string) {
        super(message)
        this.message = "Discord.GenericError: " + message;
    }
}

export class DiscordFormError extends Error {
    message: string;

    constructor(message: string) {
        super(message)
        this.message = "Discord.FormError: " + message;
    }
}

Deno.core.registerErrorClass("DiscordNotFoundError", NotFoundError);
Deno.core.registerErrorClass("DiscordPermissionsError", PermissionsError);
Deno.core.registerErrorClass("DiscordLimitReachedError", LimitReachedError);
Deno.core.registerErrorClass("DiscordServerErrorResponse", ServerError);
Deno.core.registerErrorClass("DiscordGenericErrorResponse", GenericError);
Deno.core.registerErrorClass("DiscordFormError", DiscordFormError);

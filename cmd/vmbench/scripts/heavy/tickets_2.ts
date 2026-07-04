import { Commands, Discord } from "botloader";

interface Ticket {
    id: number,
    ownerId: string,
    channelId: string | null,
    subject: string,
    category: string,
    status: "open" | "claimed" | "closed",
    claimedBy: string | null,
    createdAt: number,
    closedAt: number | null,
    messages: number,
}

const TICKET_CATEGORIES = [
    { id: "support", label: "General Support", description: "Questions about the server" },
    { id: "report", label: "User Report", description: "Report rule breaking" },
    { id: "appeal", label: "Ban Appeal", description: "Appeal a moderation action" },
    { id: "partner", label: "Partnership", description: "Partnership inquiries" },
    { id: "other", label: "Other", description: "Anything else" },
];

const tickets = script.createStorageJson<Ticket>("tickets");
const ticketCounter = script.createStorageVarNumber("ticket_counter");
const userOpenTickets = script.createStorageJson<number[]>("user_open_tickets");

const MAX_OPEN_PER_USER = 3;

function formatTicket(ticket: Ticket): string {
    const statusEmoji = ticket.status === "open" ? "🟢" : ticket.status === "claimed" ? "🟡" : "🔴";
    const lines = [
        `${statusEmoji} **Ticket #${ticket.id}** - ${ticket.subject}`,
        `category: ${ticket.category}, opened by <@${ticket.ownerId}>`,
    ];
    if (ticket.claimedBy) {
        lines.push(`claimed by <@${ticket.claimedBy}>`);
    }
    if (ticket.closedAt) {
        const openForMs = ticket.closedAt - ticket.createdAt;
        lines.push(`closed after ${Math.round(openForMs / 3_600_000)}h`);
    }
    return lines.join("\n");
}

function validCategory(id: string): boolean {
    return TICKET_CATEGORIES.some((c) => c.id === id);
}

script.createCommand(
    Commands.slashCommand("v2-ticket-open", "open a support ticket")
        .addOptionString("subject", "what the ticket is about")
        .addOptionString("category", "one of: support, report, appeal, partner, other", { required: false })
        .build(async (ctx, args) => {
            const category = args.category ?? "support";
            if (!validCategory(category)) {
                const valid = TICKET_CATEGORIES.map((c) => c.id).join(", ");
                await ctx.sendResponse(`invalid category, pick one of: ${valid}`);
                return;
            }

            const open = (await userOpenTickets.get(ctx.member.user.id))?.value ?? [];
            if (open.length >= MAX_OPEN_PER_USER) {
                await ctx.sendResponse(`you already have ${open.length} open tickets`);
                return;
            }

            const id = await ticketCounter.incr(1);
            const ticket: Ticket = {
                id,
                ownerId: ctx.member.user.id,
                channelId: null,
                subject: args.subject,
                category,
                status: "open",
                claimedBy: null,
                createdAt: Date.now(),
                closedAt: null,
                messages: 0,
            };

            await tickets.set(`${id}`, ticket);
            await userOpenTickets.set(ctx.member.user.id, [...open, id]);
            await ctx.sendResponse(`ticket #${id} opened:\n${formatTicket(ticket)}`);
        })
);

script.createCommand(
    Commands.slashCommand("v2-ticket-claim", "claim a ticket as a staff member")
        .addOptionInteger("id", "ticket number")
        .build(async (ctx, args) => {
            const entry = await tickets.get(`${args.id}`);
            if (!entry || entry.value.status === "closed") {
                await ctx.sendResponse("no open ticket with that id");
                return;
            }
            const updated: Ticket = {
                ...entry.value,
                status: "claimed",
                claimedBy: ctx.member.user.id,
            };
            await tickets.set(`${args.id}`, updated);
            await ctx.sendResponse(`you claimed ticket #${args.id}`);
        })
);

script.createCommand(
    Commands.slashCommand("v2-ticket-close", "close a ticket")
        .addOptionInteger("id", "ticket number")
        .addOptionString("reason", "why the ticket is being closed", { required: false })
        .build(async (ctx, args) => {
            const entry = await tickets.get(`${args.id}`);
            if (!entry || entry.value.status === "closed") {
                await ctx.sendResponse("no open ticket with that id");
                return;
            }

            const updated: Ticket = {
                ...entry.value,
                status: "closed",
                closedAt: Date.now(),
            };
            await tickets.set(`${args.id}`, updated);

            const open = (await userOpenTickets.get(updated.ownerId))?.value ?? [];
            await userOpenTickets.set(
                updated.ownerId,
                open.filter((v) => v !== updated.id)
            );

            await ctx.sendResponse(
                `ticket #${args.id} closed${args.reason ? `: ${args.reason}` : ""}`
            );
        })
);

script.createCommand(
    Commands.slashCommand("v2-ticket-info", "show details about a ticket")
        .addOptionInteger("id", "ticket number")
        .build(async (ctx, args) => {
            const entry = await tickets.get(`${args.id}`);
            if (!entry) {
                await ctx.sendResponse("no ticket with that id");
                return;
            }
            await ctx.sendResponse(formatTicket(entry.value));
        })
);

script.createCommand(
    Commands.slashCommand("v2-ticket-list", "list open tickets")
        .addOptionString("category", "only show this category", { required: false })
        .build(async (ctx, args) => {
            const all = await tickets.list({ limit: 100 });
            const open = all.filter((e) => e.value.status !== "closed");
            const filtered = args.category
                ? open.filter((e) => e.value.category === args.category)
                : open;

            if (filtered.length === 0) {
                await ctx.sendResponse("no open tickets");
                return;
            }

            const lines = filtered.map(
                (e) => `#${e.value.id} [${e.value.category}] ${e.value.subject} (<@${e.value.ownerId}>)`
            );
            await ctx.sendResponse(lines.join("\n"));
        })
);

script.on("MESSAGE_CREATE", async (msg) => {
    if (!msg.author || msg.author.bot) {
        return;
    }

    const all = await tickets.list({ limit: 100 });
    const ticket = all.find((e) => e.value.channelId === msg.channelId && e.value.status !== "closed");
    if (ticket) {
        await tickets.set(ticket.key, { ...ticket.value, messages: ticket.value.messages + 1 });
    }
});

script.onInterval("v2_ticket_auto_close", "0 * * * *", async () => {
    const all = await tickets.list({ limit: 100 });
    const cutoff = Date.now() - 7 * 24 * 3_600_000;
    for (const entry of all) {
        if (entry.value.status !== "closed" && entry.value.createdAt < cutoff) {
            await tickets.set(entry.key, {
                ...entry.value,
                status: "closed",
                closedAt: Date.now(),
            });
            console.log(`auto-closed stale ticket #${entry.value.id}`);
        }
    }
});

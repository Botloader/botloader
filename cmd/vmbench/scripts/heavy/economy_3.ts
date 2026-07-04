import { Commands } from "botloader";

interface ShopItem {
    id: string,
    name: string,
    description: string,
    price: number,
    role?: string,
    consumable: boolean,
}

const SHOP_ITEMS: ShopItem[] = [
    { id: "fishing_rod", name: "Fishing Rod", description: "Lets you use /fish", price: 500, consumable: false },
    { id: "shovel", name: "Shovel", description: "Lets you use /dig", price: 750, consumable: false },
    { id: "laptop", name: "Laptop", description: "Lets you use /hack", price: 2000, consumable: false },
    { id: "padlock", name: "Padlock", description: "Protects your wallet from one robbery", price: 300, consumable: true },
    { id: "lucky_coin", name: "Lucky Coin", description: "Doubles your next gamble win", price: 1000, consumable: true },
    { id: "energy_drink", name: "Energy Drink", description: "Halves work cooldown once", price: 250, consumable: true },
    { id: "trophy", name: "Trophy", description: "A shiny trophy to show off", price: 10000, consumable: false },
    { id: "ring", name: "Ring", description: "For proposing to someone", price: 5000, consumable: true },
    { id: "pet_rock", name: "Pet Rock", description: "It does nothing, but it's yours", price: 100, consumable: false },
    { id: "gold_bar", name: "Gold Bar", description: "A stable investment", price: 7500, consumable: false },
];

const WORK_JOBS = [
    { name: "programmer", min: 100, max: 400, flavor: "You fixed a production bug at 3am" },
    { name: "chef", min: 80, max: 250, flavor: "You cooked a five course meal" },
    { name: "taxi driver", min: 50, max: 200, flavor: "You drove someone across town" },
    { name: "streamer", min: 10, max: 800, flavor: "You went viral (or didn't)" },
    { name: "plumber", min: 120, max: 300, flavor: "You unclogged the unspeakable" },
    { name: "barista", min: 60, max: 180, flavor: "You drew art in the foam" },
    { name: "electrician", min: 150, max: 350, flavor: "You didn't get zapped today" },
    { name: "gardener", min: 40, max: 160, flavor: "You mowed 12 lawns" },
];

const wallets = script.createStorageNumber("eco_wallets");
const banks = script.createStorageNumber("eco_banks");
const inventories = script.createStorageJson<Record<string, number>>("eco_inventories");
const cooldownStore = script.createStorageJson<number>("eco_cooldowns");

const DAILY_AMOUNT = 500;
const DAILY_COOLDOWN_MS = 24 * 60 * 60 * 1000;
const WORK_COOLDOWN_MS = 60 * 60 * 1000;

async function checkCooldown(userId: string, action: string, durationMs: number): Promise<number> {
    const key = `${action}:${userId}`;
    const last = (await cooldownStore.get(key))?.value ?? 0;
    const remaining = last + durationMs - Date.now();
    if (remaining > 0) {
        return remaining;
    }
    await cooldownStore.set(key, Date.now());
    return 0;
}

function formatDuration(ms: number): string {
    const minutes = Math.ceil(ms / 60_000);
    if (minutes < 60) {
        return `${minutes} minutes`;
    }
    return `${Math.floor(minutes / 60)}h ${minutes % 60}m`;
}

function findItem(idOrName: string): ShopItem | undefined {
    const lowered = idOrName.toLowerCase();
    return SHOP_ITEMS.find(
        (item) => item.id === lowered || item.name.toLowerCase() === lowered
    );
}

script.createCommand(
    Commands.slashCommand("v3-balance", "check your wallet and bank balance")
        .build(async (ctx) => {
            const wallet = (await wallets.get(ctx.member.user.id))?.value ?? 0;
            const bank = (await banks.get(ctx.member.user.id))?.value ?? 0;
            await ctx.sendResponse(`wallet: ${wallet} coins, bank: ${bank} coins`);
        })
);

script.createCommand(
    Commands.slashCommand("v3-daily", "claim your daily coins")
        .build(async (ctx) => {
            const remaining = await checkCooldown(ctx.member.user.id, "daily", DAILY_COOLDOWN_MS);
            if (remaining > 0) {
                await ctx.sendResponse(`come back in ${formatDuration(remaining)}`);
                return;
            }
            await wallets.incr(ctx.member.user.id, DAILY_AMOUNT);
            await ctx.sendResponse(`you claimed ${DAILY_AMOUNT} coins!`);
        })
);

script.createCommand(
    Commands.slashCommand("v3-work", "work a random job for coins")
        .build(async (ctx) => {
            const remaining = await checkCooldown(ctx.member.user.id, "work", WORK_COOLDOWN_MS);
            if (remaining > 0) {
                await ctx.sendResponse(`you're tired, rest for ${formatDuration(remaining)}`);
                return;
            }
            const job = WORK_JOBS[Math.floor(Math.random() * WORK_JOBS.length)];
            const earned = job.min + Math.floor(Math.random() * (job.max - job.min));
            await wallets.incr(ctx.member.user.id, earned);
            await ctx.sendResponse(`${job.flavor} as a ${job.name} and earned ${earned} coins`);
        })
);

script.createCommand(
    Commands.slashCommand("v3-deposit", "move coins from wallet to bank")
        .addOptionInteger("amount", "how much to deposit")
        .build(async (ctx, args) => {
            const wallet = (await wallets.get(ctx.member.user.id))?.value ?? 0;
            if (args.amount <= 0 || args.amount > wallet) {
                await ctx.sendResponse("you don't have that much in your wallet");
                return;
            }
            await wallets.incr(ctx.member.user.id, -args.amount);
            await banks.incr(ctx.member.user.id, args.amount);
            await ctx.sendResponse(`deposited ${args.amount} coins`);
        })
);

script.createCommand(
    Commands.slashCommand("v3-withdraw", "move coins from bank to wallet")
        .addOptionInteger("amount", "how much to withdraw")
        .build(async (ctx, args) => {
            const bank = (await banks.get(ctx.member.user.id))?.value ?? 0;
            if (args.amount <= 0 || args.amount > bank) {
                await ctx.sendResponse("you don't have that much in the bank");
                return;
            }
            await banks.incr(ctx.member.user.id, -args.amount);
            await wallets.incr(ctx.member.user.id, args.amount);
            await ctx.sendResponse(`withdrew ${args.amount} coins`);
        })
);

script.createCommand(
    Commands.slashCommand("v3-pay", "send coins to another user")
        .addOptionString("user_id", "who to pay")
        .addOptionInteger("amount", "how much to send")
        .build(async (ctx, args) => {
            const wallet = (await wallets.get(ctx.member.user.id))?.value ?? 0;
            if (args.amount <= 0 || args.amount > wallet) {
                await ctx.sendResponse("you can't afford that");
                return;
            }
            await wallets.incr(ctx.member.user.id, -args.amount);
            await wallets.incr(args.user_id, args.amount);
            await ctx.sendResponse(`sent ${args.amount} coins to <@${args.user_id}>`);
        })
);

script.createCommand(
    Commands.slashCommand("v3-shop", "browse the item shop")
        .build(async (ctx) => {
            const lines = SHOP_ITEMS.map(
                (item) => `**${item.name}** (${item.price} coins) - ${item.description}`
            );
            await ctx.sendResponse(lines.join("\n"));
        })
);

script.createCommand(
    Commands.slashCommand("v3-buy", "buy an item from the shop")
        .addOptionString("item", "item id or name")
        .addOptionInteger("quantity", "how many to buy", { required: false })
        .build(async (ctx, args) => {
            const item = findItem(args.item);
            if (!item) {
                await ctx.sendResponse("no such item, check /shop");
                return;
            }

            const quantity = Math.max(1, args.quantity ?? 1);
            const cost = item.price * quantity;
            const wallet = (await wallets.get(ctx.member.user.id))?.value ?? 0;
            if (cost > wallet) {
                await ctx.sendResponse(`you need ${cost} coins but only have ${wallet}`);
                return;
            }

            await wallets.incr(ctx.member.user.id, -cost);
            const inv = (await inventories.get(ctx.member.user.id))?.value ?? {};
            inv[item.id] = (inv[item.id] ?? 0) + quantity;
            await inventories.set(ctx.member.user.id, inv);
            await ctx.sendResponse(`bought ${quantity}x ${item.name} for ${cost} coins`);
        })
);

script.createCommand(
    Commands.slashCommand("v3-inventory", "see what you own")
        .build(async (ctx) => {
            const inv = (await inventories.get(ctx.member.user.id))?.value ?? {};
            const lines = Object.entries(inv)
                .filter(([, count]) => count > 0)
                .map(([id, count]) => {
                    const item = findItem(id);
                    return `${count}x ${item?.name ?? id}`;
                });
            await ctx.sendResponse(lines.join("\n") || "your inventory is empty");
        })
);

script.createCommand(
    Commands.slashCommand("v3-rich", "leaderboard of the richest users")
        .build(async (ctx) => {
            const top = await wallets.sortedList("Descending", { limit: 10 });
            const lines = top.map((e, i) => `${i + 1}. <@${e.key}> - ${e.value} coins`);
            await ctx.sendResponse(lines.join("\n") || "everyone is broke");
        })
);

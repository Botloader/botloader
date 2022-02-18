import { ScheduledTask } from "./generated/events/ScheduledTask";
import { OpWrappers } from "./op_wrappers";

/**
 * Tasks or "Scheduled" Tasks are tasks that will execute at some point in the future
 * 
 * This can be used for a variety of functions but a simple example would be a timed ban.
 * when someone issues the timed ban you could schedule a task to unban them again at the desired time. 
 */
export namespace Tasks {

    /**
     * Create a new scheduled task.
     * 
     * @param namespace The namespace for this task, register a handler for the namespace using {@link Script.registerTaskHandler}
     * @param execute_at When to execute this task
     * @param opts Additional optional options, see {@link CreateOptions} for more info.
     * @returns The scheduled task
     */
    export async function scheduleTask(namespace: string, execute_at: Date, opts?: CreateOptions): Promise<ScheduledTask> {
        return OpWrappers.tasks.scheduleTask({
            namespace: namespace,
            executeAt: execute_at.getTime(),
            data: opts?.data ?? null,
            uniqueKey: opts?.key,
        });
    }

    export interface CreateOptions {
        /**
         * A namespaced unique key (unique to tasks with the same "name")
         * 
         * If you try to schedule 2 tasks with the same name and key then the latter one will overwrite the first
         * 
         * An example use case for this could be a "unmute" task, you could use their user id as the `key` so that only
         * one unmute task is scheduled per user, and if another one is attempted then it will just overwrite the existing one, effectively updating it.
         * 
         * This also allows you to use {@link deleteByKey} and {@link getByKey}.
         * 
         * This is optional.
         */
        key?: string,

        /**
         * Optional data to pass to the task
         * 
         * Note that this will be json encoded, so don't pass in a class and expect a class back.
         * 
         * A 1KB size limit is also imposed
         */
        data?: any,
    }

    /**
     * Delete a task by its globally unique ID (NOT key)
     * @returns true if found and deleted, false otherwise
     */
    export async function deleteById(id: number): Promise<boolean> {
        return OpWrappers.tasks.delTask(id);
    }

    /**
     * Delete a task by its namespaced unique key
     * @returns true if found and deleted, false otherwise
     */
    export async function deleteByKey(namespace: string, key: string): Promise<boolean> {
        return OpWrappers.tasks.delTaskByKey(namespace, key);
    }

    /**
     * Delete all tasks within a namespace
     * @returns the number of tasks deleted
     */
    export async function deleteNamespace(namespace: string): Promise<number> {
        return OpWrappers.tasks.delAllTasks(namespace);
    }

    /**
     * Retrieve a task by its globally unique ID (NOT uniqueKey)
     * @returns The task if found, or undefined if not found
     */
    export async function getById(id: number): Promise<ScheduledTask | undefined> {
        return await OpWrappers.tasks.getTask(id) ?? undefined;
    }

    /**
     * Retrieve a task by its namespaced unique key
     * @returns Either the task if found, or undefined  
     */
    export async function getByKey(namespace: string, key: string): Promise<ScheduledTask | undefined> {
        return await OpWrappers.tasks.getTaskByKey(namespace, key) ?? undefined;
    }

    /**
     * Paginate through all scheduled tasks, optionally filtered by namespace
     * 
     * Note that only a small number is returned each call (25 as of writing, may change) so
     * if you need to scan through all of them you need to paginate using {@link ListOptions.afterId}
     * 
     * Entries are sorted by increasing ID
     * 
     * @returns 
     */
    export async function getMany(options?: ListOptions): Promise<ScheduledTask[]> {
        return OpWrappers.tasks.getAllTasks(options?.namespace, options?.afterId ?? 0);
    }

    export interface ListOptions {
        /**
         * Optionally filter by namespace
         */
        namespace?: string,

        /**
         * Optionally only return items after the provided id
         * 
         * You can use this for pagination
         */
        afterId?: number,
    }

    export interface Task<T> extends ScheduledTask {
        data: T;
    }
}


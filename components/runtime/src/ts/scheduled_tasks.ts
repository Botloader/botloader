import { OpWrappers } from "./op_wrappers";
import type { ScheduledTask as InternalTask, GetGuildTasksFilter } from "./generated/internal/index";

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
     * @deprecated does not support plugins
     * 
     * @param namespace The namespace for this task, register a handler for the namespace using {@link Script.registerTaskHandler}
     * @param execute_at When to execute this task
     * @param opts Additional optional options, see {@link CreateOptions} for more info.
     * @returns The scheduled task
     */
    export async function schedule(namespace: string, execute_at: Date, opts?: Omit<CreateOptions<any>, "executeAt">): Promise<ScheduledTask> {
        const task = await OpWrappers.tasks.scheduleTask({
            pluginId: null,
            namespace: namespace,
            executeAt: execute_at.getTime(),
            data: opts?.data ?? null,
            uniqueKey: opts?.key,
        });

        return convertInternalTask(task)
    }

    export type CreateOptions<T = undefined> = {
        executeAt: Date,

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
    } & (T extends undefined ? CreateOptionsDataOptional<T> : CreateOptionsDataRequired<T>)

    interface CreateOptionsDataRequired<T> {
        /**
         * Optional data to pass to the task
         * 
         * Note that this will be json encoded, so don't pass in a class and expect a class back.
         * 
         * A 1KB size limit is also imposed
         */
        data: T
    }

    interface CreateOptionsDataOptional<T> {
        /**
         * Optional data to pass to the task
         * 
         * Note that this will be json encoded, so don't pass in a class and expect a class back.
         * 
         * A 1KB size limit is also imposed
         */
        data?: T
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
     * 
     * @deprecated does not support plugins
     */
    export async function deleteByKey(namespace: string, key: string): Promise<boolean> {
        return OpWrappers.tasks.delTaskByKey(null, namespace, key);
    }

    /**
     * Delete all tasks within a namespace
     * @returns the number of tasks deleted
     * 
     * @deprecated does not support plugins
     */
    export async function deleteNamespace(namespace: string): Promise<number> {
        return OpWrappers.tasks.delAllTasks(null, namespace);
    }

    /**
     * Retrieve a task by its globally unique ID (NOT uniqueKey)
     * @returns The task if found, or undefined if not found
     */
    export async function getById(id: number): Promise<ScheduledTask | undefined> {
        const task = await OpWrappers.tasks.getTask(id) ?? undefined;
        if (task) {
            return convertInternalTask(task)
        }
    }

    /**
     * Retrieve a task by its namespaced unique key
     * @returns Either the task if found, or undefined  
     * 
     * @deprecated does not support plugins
     */
    export async function getByKey(namespace: string, key: string): Promise<ScheduledTask | undefined> {
        const task = await OpWrappers.tasks.getTaskByKey(null, namespace, key) ?? undefined;
        if (task) {
            return convertInternalTask(task)
        }
    }

    /**
     * Paginate through all scheduled tasks, optionally filtered by namespace
     * 
     * Note that only a small number is returned each call (25 as of writing, may change) so
     * if you need to scan through all of them you need to paginate using {@link ListOptions.afterId}
     * 
     * Entries are sorted by increasing ID
     * 
     */
    export async function getMany(options?: ListOptions): Promise<ScheduledTask[]> {
        const tasks = await OpWrappers.tasks.getAllTasks({
            namespace: options?.namespace ?? null,
            scope: { kind: "Guild" },
        }, options?.afterId ?? 0);

        return tasks.map(v => convertInternalTask(v))
    }

    export interface ListOptions {
        /**
         * Optionally filter by namespace
         */
        namespace?: string,

        // /**
        //  * Optionally filter by plugin
        //  * 
        //  * Setting this to null filters out all plugins
        //  */
        // plugin_id?: string | null,

        /**
         * Optionally only return items after the provided id
         * 
         * You can use this for pagination
         */
        afterId?: number,
    }

    export interface BucketListOptions {
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

    export interface ScheduledTask {
        id: number;
        pluginId: string | null,
        namespace: string;
        key?: string;
        executeAt: number;
        data: unknown;
    }

    interface TaskBucketOptions {
        pluginId: string | null,
        name: string,
    }

    export class TaskBucket<T = any> {
        pluginId: string | null;
        name: string;

        constructor(options: TaskBucketOptions) {
            this.pluginId = options.pluginId
            this.name = options.name
        }


        /**
        * Create a new scheduled task.
        * 
        * @returns The scheduled task
        */
        async schedule(opts: CreateOptions<T>): Promise<Task<T>> {
            const task = await OpWrappers.tasks.scheduleTask({
                pluginId: this.pluginId,
                namespace: this.name,
                executeAt: opts.executeAt.getTime(),
                data: opts.data ?? null,
                uniqueKey: opts.key,
            });

            return convertInternalTask(task)
        }

        /**
         * Retrieve a task by its globally unique ID (NOT unique key)
         * 
         * @returns The task if found, or undefined if not found
         */
        async getById(id: number): Promise<Task<T> | undefined> {
            const task = await OpWrappers.tasks.getTask(id) ?? undefined;
            if (task) {
                const converted = convertInternalTask<T>(task)
                if (converted.namespace !== this.name || converted.pluginId !== this.pluginId) {
                    throw new Error("Retrieved task does not belong to bucket")
                }

                return converted
            }
        }

        /**
         * Delete a task by its globally unique ID (NOT key)
         * 
         * Throws an error if it did not belong to this bucket
         * 
         * @returns true if found and deleted, false otherwise
         */
        async deleteById(id: number): Promise<boolean> {
            // Ensure this entry is from this bucket
            await this.getById(id)

            return OpWrappers.tasks.delTask(id);
        }

        /**
         * Delete a task by its unique key
         * 
         * @returns true if found and deleted, false otherwise
         */
        async deleteByKey(key: string): Promise<boolean> {
            return OpWrappers.tasks.delTaskByKey(this.pluginId, this.name, key);
        }

        /**
         * Delete all tasks within this bucket
         * 
         * @returns the number of tasks deleted
         */
        async deleteAll(): Promise<number> {
            return OpWrappers.tasks.delAllTasks(this.pluginId, this.name);
        }

        /**
         * Retrieve a task by its unique key
         * 
         * @returns Either the task if found, or undefined  
         */
        async getByKey(key: string): Promise<Task<T> | undefined> {
            const task = await OpWrappers.tasks.getTaskByKey(this.pluginId, this.name, key) ?? undefined;
            if (task) {
                return convertInternalTask(task)
            }
        }

        /**
         * Paginate through all scheduled tasks in this bucket
         * 
         * Note that only a small number is returned each call (25 as of writing, may change) so
         * if you need to scan through all of them you need to paginate using {@link BucketListOptions.afterId}
         * 
         * Entries are sorted by increasing ID
         */
        async getMany(options?: BucketListOptions): Promise<Task<T>[]> {
            const tasks = await OpWrappers.tasks.getAllTasks({
                namespace: this.name,
                scope: this.pluginId
                    ? { kind: "Plugin", plugin_id: this.pluginId }
                    : { kind: "Guild" }
            }, options?.afterId ?? 0);

            return tasks.map(v => convertInternalTask(v))
        }

    }

    function convertInternalTask<T>(task: InternalTask): Task<T> {
        return {
            ...task,
            data: task.data as T,
        }
    }
}

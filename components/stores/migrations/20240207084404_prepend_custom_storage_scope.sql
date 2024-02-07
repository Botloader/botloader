-- update plugins dev source
UPDATE
    plugins
SET
    script_dev_source = '// This section was automatically patched by a migration tool.
// Botloader recently changed plugins to have storage be put in another scope by default to avoid name clashing with guild scripts.
// The below code changes this back to the default behavior so that your plugin may continue to work as it used to before without having your storage effectively wiped.
// There will be a way to migrate storage into the plugin scope at some point.
script.setCustomStorageScope({kind: "Guild"})

' || script_dev_source WHERE script_dev_source LIKE '%.createStorage%';

-- update plugins published source
UPDATE
    plugins
SET
    script_published_source = '// This section was automatically patched by a migration tool.
// Botloader recently changed plugins to have storage be put in another scope by default to avoid name clashing with guild scripts.
// The below code changes this back to the default behavior so that your plugin may continue to work as it used to before without having your storage effectively wiped.
// There will be a way to migrate storage into the plugin scope at some point.
script.setCustomStorageScope({kind: "Guild"})

' || script_published_source WHERE script_published_source LIKE '%.createStorage%';

-- update attached plugins script source
UPDATE
    guild_scripts
SET
    original_source = '// This section was automatically patched by a migration tool.
// Botloader recently changed plugins to have storage be put in another scope by default to avoid name clashing with guild scripts.
// The below code changes this back to the default behavior so that your plugin may continue to work as it used to before without having your storage effectively wiped.
// There will be a way to migrate storage into the plugin scope at some point.
script.setCustomStorageScope({kind: "Guild"})

' || original_source WHERE original_source LIKE '%.createStorage%'
    AND plugin_id IS NOT NULL;


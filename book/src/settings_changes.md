## Changing the "name" of a option

If you change the "name" of a option (not the label), then you will effectively wipe the saved value for it. This is because the name act as the "key" or "id" of the option. The label field exists so you can give a different cosmetic label to the option.

## Changing a list object schema

Changing the list object schema can result in bad entries residing in a list, if you added a new required option the botloader will automatically filter out those bad entries when you load the value.

## Reusing the option value name

Reusing the option value name for oen of more options is undefine behavior, i have plans on allowing scripts to change settings and this will break if you do so.

## Changing option type

Botloader will try to do a best effort to maintain the typing correctness, it will wipe values if needed.

## Changing option validation settings

As of writing, botloader does not re-validate saved options when you do this, so this could result in unexpected values if you do this. This may change in the future as the bot matures.
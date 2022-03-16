# Storage

Your vm is constantly restarted on botloader, this is to save on resources when it's not being used. Because of this there's a storage API for having persistent storage.

This storage APi give you a key value database where you can store things such as numbers, or arbitrary json encoded data.

## Buckets

This API is centered around `buckets`, a bucket can be thought of as a namespace, the same key can hold different values in different buckets even if the key is the same.

The same bucket can also be used in multiple scripts safely, allowing you to share data between them without any problems.

As of writing there are 2 different kind of buckets:
 - Json objet buckets [API](https://botloader.io/docs/classes/Script.html#createGuildStorageJson)
 - Number buckets [API](https://botloader.io/docs/classes/Script.html#createGuildStorageNumber)


## Json buckets

Json buckets can hold any kind of object that can be used in `JSON.stringify`. Botloader handles the encoding and decoding from and to json for you.

**See the API docs for Json buckets for all the methods available:** [here](https://botloader.io/docs/classes/Storage.JsonBucket.html)

## Number buckets

Number buckets are a bit special, since they only hold numbers they can be sorted efficiently behind the scenes, because of this they have the ability to return a list of sorted entries by their value.

An example of this would be a bucket for user scores where the key is the users id and the value is their score. Using this you can easily fetch the top entries for display in a leader-board without having to worry about sorting yourself.

**See the API docs for Number buckets for all the methods available:** [here](https://botloader.io/docs/classes/Storage.NumberBucket.html)
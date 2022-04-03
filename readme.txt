-------------------------------
Limit Order Book Reconciliation
-------------------------------

This program creates and maintains an up-to-date Limit Order Book for the BTC-PERPETUAL future, according to data
provided by deribit.com and prints the best bids/asks prices and quantities pairs every second. On packet loss, we
reconnect with the server.

-------------------------------
    Implementation Choices
-------------------------------

1. The websocket
For the websocket I utilized the tokio-tungstenite crate, an asynchronous lightweight websocket. Having everything
non-blocking and instead waiting/running on the background, makes everything faster. We also use methods provided by
tokio for the printing function. The whole main function is encased in a loop. In case of packet loss, we break from the
current cycle and use continue to go to the next one that performs the websocket connection from the start.

2. The parsing
The parsing of the data is achieved with the help of serde and serde_json crates. After we get a message from the
stream, we deserialize it with the aforementioned crates into structs, that we then use to perform the necessary
operations. The only bad practice in this project, is that every struct is public and the same is true for their fields.
Usually some form of encapsulation should be implemented, but since each message struct contains nested structs that
contain nested structs and vectors, instead of writing a plethora of setters/getters we have everything public. If
Rust was a true oop language, under no circumstances would we have done this.

3. The collection
For the book we have a struct containing 4 values (the highest bid,the lowest ask and their quantities) as well as 2
Hashmaps in which we store all of them. Hashmaps gives us the best complexity O(1) for storing,returning or deleting a
value and we do all of these operations very frequently. Unfortunately we don't have a tree's ability to get the impo-
rtant pairs, but nothing stops us from storing those in our struct.

-------------------------------
        Program Rundown
-------------------------------
We connect to deribit.com and afterwards we split the stream into write and read. Write sends the subscription request
and read gets the notifications stream. We parse the first one and use it to create the Book. From there we create a
loop and inside the loop we use the tokio::select! macro, which allows us to wait on multiple async computations. The
two computations are transforming the Book and printing of the necessary values every second. For the transformations,
as with the book, we deserialize the notification and if everything is good we pass it along with a reference of the
book for transformation. First thing we do is comparing the previous_change_id of the change with the Books id. If we
have encountered packet loss and those two do not much we clear the snapshot and we return a value that breaks the
current loop of the reading as well as the current connection and we reconnect. If everything is good, we start itera-
ting through each change, first for the bids then for the asks. If we need to delete a pair we also check if it is the
value we need to print and if it is we use a flag to notify that at the end of all the changes we need to update that
field. If the event requires us to create a new value or change an existing one we check if the value is equal or higher
of the existing one and we accordingly insert or change a pair. The process is performed for the asks. After everything
we have done we check to see if one of the extreme values was deleted and needs updating and we do just that if needed.
And that's that. If nothing goes wrong the program continues to read from the stream, transform the book and print to
stdout all asynchronously.

-------------------------------
      Running the program
-------------------------------

Just click run (or type cargo run). The program does not implement a graceful exit and runs for as long as people
continue to buy and sell perpetual contracts for Bitcoin on Deribit.

## Book search example for Python SDK

An example on how to use the python sdk that shows the process fo splitting and 
inserting an epub ebook into the db and querying it via a search phrase either directly or contextually

To isntall dependencies (ensure you have poetry installed)  
```poetry install```

To insert run  
```poetry run insertbook```

To search run  
```poetry run searchbook```

Note that the epub file being split is available locally in the example file and you can edit the example to customize processes and play around with input and output.

Here's a GIF of insertion of the book
![insertion gif](insertbook.gif)
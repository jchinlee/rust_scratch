RC = rustc
FLAGS = 
RM = rm

SRC = .

all: server

server:
	$(RC) $(SRC)/server.rs
	$(RC) $(SRC)/client.rs

clean:
	$(RM) -rf ./*.d*
	$(RM) $(SRC)/server
	$(RM) $(SRC)/client

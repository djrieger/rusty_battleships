import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

Item {
    id: screen

    anchors.fill: parent
    visible: false

    // TODO: provide button for surrender
    signal gameEnded();

    ListModel {
        id: shipModel

		ListElement {
			name: "Aircraft carrier"
			length: 5
			hp: 5
			x: -1
			y: -1
			horizontal: false
			reverse: false
		}
		ListElement {
			name: "Battleship"
			length: 4
			hp: 4
			x: -1
			y: -1
			horizontal: false
			reverse: false
		}
		ListElement {
			name: "Cruiser"
			length: 3
			hp: 3
			x: -1
			y: -1
			horizontal: false
			reverse: false
		}
		ListElement {
			name: "Destroyer"
			length: 2
			hp: 2
			x: -1
			y: -1
			horizontal: false
			reverse: false
		}
		ListElement {
			name: "Submarine"
			length: 2
			hp: 2
			x: -1
			y: -1
			horizontal: false
			reverse: false
		}
    }

	ColumnLayout {
	    RowLayout {
            Rectangle {
                id: board

                width: 200; height: 200;
                color: active ? (placement_phase ? "yellow" : (moveAllowed ? "green" : "blue")) : "red"

                property int currentX: -1
                property int currentY: -1

                property bool active: true
                property bool moveAllowed: false
                property int moveDirection: -1
                property int moveShip: -1
                property bool placement_phase: true

                Grid {
                    anchors.fill: parent

                    x: 5; y: 5
                    rows: 10; columns: 10; spacing: 1

                    Repeater {
                        id: boardButtons

                        model: 100

                        Rectangle {
                            width: parent.width / parent.columns - parent.spacing
                            height: parent.height / parent.rows - parent.spacing

                            property string text: " "
                            property string textColor: "black"
                            property bool revealed: false

                            color: revealed ? "skyblue" : "white"

                            Text {
                                text: parent.text
                                color: parent.textColor
                                font.pixelSize: Math.round(parent.height * 0.8)
                                anchors.centerIn: parent
                            }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: {
                                    board_clicked(index);
                                }
                            }
                        }
                    }
                }
            }

            Rectangle {
                width: 200; height: 200; color: "blue"

                Grid {
                    anchors.fill: parent

                    x: 5; y: 5
                    rows: 10; columns: 10; spacing: 1

                    Repeater {
                        id: opponentBoardButtons

                        model: 100

                        Rectangle {
                            property string text: "?"
                            width: parent.width / parent.columns - parent.spacing
                            height: parent.height / parent.rows - parent.spacing

                            Text {
                                text: parent.text
                                font.pixelSize: Math.round(parent.height * 0.8)
                                anchors.centerIn: parent
                            }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: {
                                    opp_board_clicked(index);
                                }
                            }
                        }
                    }
                }
            }
        }

        RowLayout {
            Button {
                width: 10
                height: 10
                text: "∧"
                enabled: board.moveAllowed && board.moveShip !== -1
                onClicked: {
                    try_move(0);
                }
            }
            Button {
                width: 10
                height: 10
                text: ">"
                enabled: board.moveAllowed && board.moveShip !== -1
                onClicked: {
                    try_move(1);
                }
            }
            Button {
                width: 10
                height: 10
                text: "∨"
                enabled: board.moveAllowed && board.moveShip !== -1
                onClicked: {
                    try_move(2);
                }
            }
            Button {
                width: 10
                height: 10
                text: "<"
                enabled: board.moveAllowed && board.moveShip !== -1
                onClicked: {
                    try_move(3);
                }
            }
        }

        RowLayout {
            TableView {
                TableViewColumn {
                    role: "name"
                    title: "Ship"
                    width: 100
                }
                TableViewColumn {
                    role: "hp"
                    title: "Hit points"
                    width: 80
                }
                model: shipModel
            }

            ColumnLayout {
                Text {
                    id: hitCounter
                    property int count: 0
                    text: "Hits: " + count + "/16"
                }
                Text {
                    id: destroyedCounter
                    property int count: 0
                    text: "Destroyed: " + count + "/5"
                }
            }
        }
	}

    function opp_board_clicked(index) {
        if (board.moveAllowed) {
            // did not complete move, reset selected ship
            board.moveShip = -1;
        }

        if (board.active && !board.placement_phase) {
	        var x = index % 10;
	        var y = Math.floor(index / 10);
	        bridge.move_and_shoot(x, y, board.moveShip, board.moveDirection);
	        board.moveAllowed = false;
	        board.active = false;
        }
    }

    function board_clicked(index) {
        if (board.active) {
            if (board.placement_phase) {
                // set coordinates or handle placement on second click
                if (board.currentX == -1) {
                    board.currentX = index % 10;
                    board.currentY = Math.floor(index / 10);
                } else {
		            var x = index % 10;
		            var y = Math.floor(index / 10);

		            handle_placement(x, y);

		            board.currentX = -1;
		            board.currentY = -1;
                }
            } else {
                board.moveShip = bridge.get_ship_at(index % 10, Math.floor(index / 10));
            }
        }
    }

    function handle_placement(x, y) {
		var xDiff = x - board.currentX;
		var yDiff = y - board.currentY;
		var shipId = -1;
		var horizontal = true;
		var reverse = false;

		if (xDiff == 0) {
			var length = Math.abs(yDiff) + 1;
			reverse = yDiff < 0;
			horizontal = false;

			if (length == 2) {
				// special case: 2 ships with length 2
		        if (shipModel.get(3).x == -1) {
			        shipId = 3;
			    } else {
			        shipId = 4;
			    }
			} else if (length > 2 && length < 6) {
			    shipId = 5 - length;
			} else {
			    // TODO: invalid length, show error
			    console.log("Invalid ship length");
			}
		} else if (yDiff == 0) {
			var length = Math.abs(xDiff) + 1;
			reverse = xDiff < 0;

			if (length == 2) {
				// special case: 2 ships with length 2
			    if (shipModel.get(3).x == -1) {
			        shipId = 3;
			    } else {
			        shipId = 4;
			    }
			} else if (length > 2 && length < 6) {
			    shipId = 5 - length;
			} else {
			    // TODO: invalid length, show error
		  	    console.log("Invalid ship length");
			}
		} else {
			// TODO: diagonal, show error
			console.log("Diagonal ship");
		}

		if (shipId != -1) {
			if (shipModel.get(shipId).x != -1) {
			    // TODO: double placement, show error
			    console.log("Double placement");
			} else {
			    shipModel.get(shipId).x = board.currentX;
			    shipModel.get(shipId).y = board.currentY;
			    shipModel.get(shipId).horizontal = horizontal;
			    shipModel.get(shipId).reverse = reverse;

			    draw_ship(shipId);
			}
		}

		if ([0, 1, 2, 3, 4].filter(function(i) { return shipModel.get(i).x === -1; }).length === 0) {
		    board.active = false;
		    board.placement_phase = false;
		    var placement = [];
		    for (var i = 0; i < 5; i++) {
		        placement.push(shipModel.get(i));
		    }
		    bridge.handle_placement(JSON.stringify(placement));
		}
    }

    function draw_ship(index) {
        console.assert(index >= 0 && index < 5);

        var ship = shipModel.get(index);

        if (ship.hp === 0) {
            return;
        }

        var buttonIndex = 10 * ship.y + ship.x;

        for (var i = 0; i < ship.length; i++) {
	        var button = boardButtons.itemAt(buttonIndex);

	        // special case for submarine so it's easy to identify
	        if (index === 4) {
	            button.textColor = "grey";
	        }

	        if (i == 0) {
	            if (ship.horizontal) {
	                button.text = ship.reverse ? ">" : "<";
	            } else {
	                button.text = ship.reverse ? "∨" : "∧";
	            }
	        } else if (i == ship.length - 1) {
	            if (ship.horizontal) {
	                button.text = ship.reverse ? "<" : ">";
	            } else {
	                button.text = ship.reverse ? "∧" : "∨";
	            }
	        } else {
	            button.text = ship.horizontal ? "=" : "||";
	        }

	        if (ship.reverse) {
	            buttonIndex -= ship.horizontal ? 1 : 10;
	        } else {
	            buttonIndex += ship.horizontal ? 1 : 10;
	        }
        }
    }

    function clearBoard(resetRevealed) {
        for (var i = 0; i < 100; i++) {
            var cell = boardButtons.itemAt(i);
            cell.text = " ";
            cell.textColor = "black";
            if (resetRevealed) {
                cell.revealed = false;
            }
        }
    }

    function try_move(direction) {
        console.assert(direction > -1 && direction < 4);
        if (bridge.can_move_in_direction(board.moveShip, direction)) {
	        move(direction);
        } else {
            // TODO: show error message
            console.log("Invalid move");
        }
    }

    function move(direction) {
        board.moveAllowed = false;
		board.moveDirection = direction;

		if (direction === 0) {
			shipModel.get(board.moveShip).y--;
		} else if (direction === 1) {
			shipModel.get(board.moveShip).x++;
		} else if (direction === 2) {
            shipModel.get(board.moveShip).y++;
        } else if (direction === 3) {
            shipModel.get(board.moveShip).x--;
        }

		clearBoard();
		draw_ship(0);
		draw_ship(1);
		draw_ship(2);
		draw_ship(3);
		draw_ship(4);
    }

    function updateBoards() {
        var opp_board = bridge.get_opp_board();
        for (var i = 0; i < opp_board.length; i++) {
            opponentBoardButtons.itemAt(i).text = opp_board[i];
        }
        if (!board.placement_phase) {
	        var my_board = bridge.get_my_board_visibility();
	        for (var i = 0; i < my_board.length; i++) {
	            boardButtons.itemAt(i).revealed = my_board[i] === "1";
	        }
        }
    }

    function updateHitPoints() {
		var hitPoints = eval(bridge.get_ships_hps());
		for (var i = 0; i < 5; i++) {
            shipModel.get(i).hp = hitPoints[i];
        }

        hitCounter.count = bridge.get_hits();
        destroyedCounter.count = bridge.get_destroyed();
    }

    function updateState() {
        var state = bridge.poll_state();

        if (state === "Planning") {
            if (!board.active) {
                // Your turn started
                board.active = true;
                board.moveAllowed = true;
		        board.moveDirection = -1;
		        board.moveShip = -1;

				// re-draw ships (some might be destroyed now)
		        clearBoard();
                draw_ship(0);
                draw_ship(1);
                draw_ship(2);
                draw_ship(3);
                draw_ship(4);
            }
        } else if (state === "OpponentPlanning") {
            if (board.active) {
                // AFK received
                board.active = false;

                // reset last move, if ship was moved
                if (!board.moveAllowed) {
                    move((board.moveDirection + 2) % 4);
                }
            }
        } else if (state === "Available") {
            screen.gameEnded();
        }
    }

    function activate() {
        timer.triggered.connect(updateBoards);
        timer.triggered.connect(updateHitPoints);
        timer.triggered.connect(updateState);
        // TODO: pass opponent info and set title text accordingly
        visible = true;
    }

    function deactivate() {
        timer.triggered.disconnect(updateBoards);
        timer.triggered.disconnect(updateHitPoints);
        timer.triggered.disconnect(updateState);

        // reset board
        board.active = true;
        board.moveAllowed = false;
        board.moveDirection = -1;
        board.moveShip = -1;
        board.placement_phase = true;

        clearBoard(true);
        for (var i = 0; i < 100; i++) {
            opponentBoardButtons.itemAt(i).text = "?";
        }
        for (var i = 0; i < 5; i++) {
            var ship = shipModel.get(i);
            ship.x = -1;
            ship.hp = ship.length;
        }

        hitCounter.count = 0;
        destroyedCounter.count = 0;

        visible = false;
    }
}

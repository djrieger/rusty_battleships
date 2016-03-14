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

    RowLayout {
        Rectangle {
            id: board

            width: 200; height: 200; color: "blue"

            property int currentX: -1;
            property int currentY: -1;

            property bool active: true
            property bool placement_phase: true

            // only used during the placement phase
            property list<QtObject> placement: [
              QtObject {
                property int x: -1
                property int y: -1
                property int length: 5
                property bool horizontal
                property bool reverse
              },
              QtObject {
                property int x: -1
                property int y: -1
                property int length: 4
                property bool horizontal
                property bool reverse
              },
              QtObject {
                property int x: -1
                property int y: -1
                property int length: 3
                property bool horizontal
                property bool reverse
              },
              QtObject {
                property int x: -1
                property int y: -1
                property int length: 2
                property bool horizontal
                property bool reverse
              },
              QtObject {
                property int x: -1
                property int y: -1
                property int length: 2
                property bool horizontal
                property bool reverse
              }
            ]

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

                        Text {
                            text: parent.text //index
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
                        width: parent.width / parent.columns - parent.spacing
                        height: parent.height / parent.rows - parent.spacing

                        Text {
                            text: "?"
                            font.pixelSize: Math.round(parent.height * 0.8)
                            anchors.centerIn: parent
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: {
                                bridge.on_clicked_opp_board(index);
                            }
                        }
                    }
                }
            }
        }
    }

    ColumnLayout {
        Button {
            id: moveLeftBtn
            width: 30
            height: 30
            Text { text: "<" }
        }
        Button {
            id: moveRightBtn
            width: 30
            height: 30
            Text { text: ">" }
        }
        Button {
            id: moveUpBtn
            width: 30
            height: 30
            Text { text: "Up" }
        }
        Button {
            id: moveDownBtn
            width: 30
            height: 30
            Text { text: "Down" }
        }
        CheckBox {
            text: "Bereit"
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
          // set current coordinates for move
          // TODO: check whether ship was selected
          board.currentX = index % 10;
          board.currentY = index / 10;
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
          if (board.placement[3].x == -1) {
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
          if (board.placement[3].x == -1) {
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
        if (board.placement[shipId].x != -1) {
          // TODO: double placement, show error
          console.log("Double placement");
        } else {
          board.placement[shipId].x = board.currentX;
          board.placement[shipId].y = board.currentY;
          board.placement[shipId].horizontal = horizontal;
          board.placement[shipId].reverse = reverse;

          draw_ship(shipId);
        }
      }
    }

    function draw_ship(index) {
      console.assert(index >= 0 && index < 5);

      var ship = board.placement[index];
      var buttonIndex = 10 * ship.y + ship.x;
      for (var i = 0; i < ship.length; i++) {
        var button = boardButtons.itemAt(buttonIndex);

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
      if ([0, 1, 2, 3, 4].filter(function(i) { return board.placement[i].x === -1; }).length === 0) {
          board.placement_phase = false;
          bridge.handle_placement(JSON.stringify(board.placement)); 
      }
    }


    function activate() {
      // TODO: pass opponent info and set title text accordingly
      visible = true;
    }

    function deactivate() {
      visible = false;
    }
}

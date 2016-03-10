import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

ApplicationWindow {
    visible: true
    title: "Rusty Battleships v1.337"

    property int margin: 11
    width: mainLayout.implicitWidth + 2 * margin
    height: mainLayout.implicitHeight + 2 * margin
    minimumWidth: 800 + 2 * margin
    minimumHeight: 600 + 2 * margin

    Timer {
        interval: 50
        running: true
        repeat: true
        onTriggered: {
            statusLabel.text = bridge.poll_state();
            logLabel.text = bridge.poll_log();
        }
    }

    statusBar: StatusBar {
        RowLayout {
            anchors.fill: parent
            RowLayout {
                Label { text: "Read Only"; id: statusLabel }
                Label { text: "Read Only"; id: logLabel }
            }
        }
    }

    RowLayout {
        id: mainLayout
        anchors.fill: parent
        anchors.margins: margin

        Rectangle {
            id: fieldContainer
            color: "white"
            Layout.fillWidth: true
            Layout.fillHeight: true

            GridLayout {
                id: field
                anchors.centerIn: parent
                width: Math.min(fieldContainer.height, fieldContainer.width)
                height: width
                rows: 5
                columns: 5
            }
        }

        ListView { //TODO: Needs to be filled.
            id: userList
            width: 200
            Layout.fillHeight: true
            model: ListModel {
                ListElement {
                    name: "Captain Kirk"
                    colorCode: "lightgrey"
                }

                ListElement {
                    name: "Captain Nemo"
                    colorCode: "lime"
                }

                ListElement {
                    name: "Admiral Ackbar"
                    colorCode: "lightgrey"
                }

                ListElement {
                    name: "Captain Balou"
                    colorCode: "lightgrey"
                }
            }
            delegate: Item {
                x: 5
                width: 80
                height: 15
                Row {
                    id: row1
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 3

                    Rectangle {
                        width: 200
                        height: 15
                        color: "transparent"

                        Rectangle {
                            width: 15
                            height: 15
                            color: colorCode
                            anchors.left: parent.left

                            Text {
                                text: name
                                anchors.left: parent.right
                                font.bold: false
                            }
                        }

                        MouseArea {
                            id: mouse_area1
                            z: 1
                            hoverEnabled: true
                            anchors.fill: parent

                            onClicked:{
                                userList.currentIndex = index
                                console.log("Challenged player " + index);
                            }
                        }
                    }
                }
            }
        }
    }

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
                                // TODO: remove, just for testing:
                                bridge.get_boards();
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

    function login() {
        bridge.send_login_request(usernameField.text);
        bridge.poll_state();
    }

    function features() {
        bridge.send_get_features_request();
        bridge.poll_state();
        featuresLabel.text = bridge.get_features_list();
    }

    function update_status() {
        Statusbar.statusLabel = bridge.get_last_message();
    }

    function connect() {
        bridge.connect(hostnameField.text);
    }

    function update_lobby() {
        bridge.get_ready_players();
        bridge.get_available_players();
        //^-- verwursten in list items!
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
    }
}

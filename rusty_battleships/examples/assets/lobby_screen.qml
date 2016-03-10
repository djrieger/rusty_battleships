import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

Item {
    id: screen

    anchors.fill: parent
    visible: false

    // TODO: provide button
    signal disconnected();
    signal gameStarted();

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
                                // FIXME: do something
                                screen.gameStarted();
                            }
                        }
                    }
                }
            }
        }
    }

    function update_lobby() {
        bridge.get_ready_players();
        bridge.get_available_players();
        //^-- verwursten in list items!
    }


    function activate() {
      // TODO: pass server info and set title text accordingly
      visible = true;
    }

    function deactivate() {
      visible = false;
    }
}

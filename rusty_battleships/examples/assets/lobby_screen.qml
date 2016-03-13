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

    GridView {
        id: userList

        anchors.fill: parent

        model: ListModel {
            ListElement {
                name: "Captain Kirk"
                ready: true
            }

            ListElement {
                name: "Captain Nemo"
                ready: false
            }

            ListElement {
                name: "Admiral Ackbar"
                ready: false
            }

            ListElement {
                name: "Captain Balou"
                ready: false
            }

            ListElement {
                name: "Captain Kirk"
                ready: true
            }

            ListElement {
                name: "Captain Nemo"
                ready: false
            }

            ListElement {
                name: "Admiral Ackbar"
                ready: false
            }

            ListElement {
                name: "Captain Balou"
                ready: false
            }
            ListElement {
                name: "Captain Kirk"
                ready: true
            }

            ListElement {
                name: "Captain Nemo"
                ready: false
            }

            ListElement {
                name: "Admiral Ackbar"
                ready: false
            }

            ListElement {
                name: "Captain Balou"
                ready: false
            }
        }

        cellHeight: 50
        cellWidth: 200
        flow: GridView.FlowTopToBottom

        delegate: Button {
            antialiasing: true
            height: 40
            width: 190
            y: 10 // vertical spacing

            enabled: ready && !waitCheckbox.checked

            RowLayout {
                // some padding
                height: parent.height - 4
                width: parent.width - 9
                x: 7
                y: 2

                opacity: enabled ? 1.0 : 0.3

                Rectangle {
                    color: ready ? "green" : "red"
                    anchors.verticalCenter: parent.verticalCenter
                    border {
                        color: "black"
                        width: 1
                    }
                    height: parent.height - 10
                    width: height
                    radius: height * 0.5
                }

                ColumnLayout {
                    spacing: 1

                    Text {
                        font {
                            pointSize: 11
                            weight: Font.DemiBold
                        }
                        text: name
                    }

                    Text {
                        font {
                            italic: true
                            pointSize: 8
                        }
                        text: ready ? "ready" : "not ready"
                    }
                }
            }

            onClicked:{
                userList.currentIndex = index
                console.log("Challenged player " + index);
                // FIXME: do something
                screen.gameStarted();
            }
        }
    }

    CheckBox {
        id: waitCheckbox
        anchors.bottom: parent.bottom
        text: "Wait for challenge from another player"
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

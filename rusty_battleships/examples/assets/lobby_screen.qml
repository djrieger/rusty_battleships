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
        id: lobby

        anchors.fill: parent

        model: ListModel { id: lobbyModel }

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

            onClicked: {
                lobby.currentIndex = index;
                console.log("Challenged player " + index);
                // FIXME: actually challenge the player
                screen.gameStarted();
            }
        }
    }

    CheckBox {
        id: waitCheckbox
        anchors.bottom: parent.bottom
        text: "Wait for challenge from another player"

        onCheckedChanged: {
            // FIXME: actually change ready state
            if (checked) {
                console.log("checked");
            } else {
                console.log("unchecked");
            }
        }
    }


    function updateLobby() {
        var lobby = eval(bridge.update_lobby());
        lobbyModel.clear();
        lobby.available_players.map(function (player) {
            lobbyModel.append({
                name: player.name,
                ready: lobby.ready_players.indexOf(player.name) !== -1
            });
        });
    }


    function checkGameStarted() {
        // TODO: check whether I was challenged. If yes, start game
    }


    function activate() {
        // TODO: pass server info and set title text accordingly
        timer.triggered.connect(updateLobby);
        timer.triggered.connect(checkGameStarted);
        visible = true;
    }

    function deactivate() {
        timer.triggered.disconnect(updateLobby);
        timer.triggered.connect(checkGameStarted);
        visible = false;
    }
}

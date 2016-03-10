import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

ApplicationWindow {
    id: connectWindow

    visible: true
    title: "Verbinden"

    property int margin: 11
    minimumWidth: 400 + 2 * margin
    minimumHeight: 300 + 2 * margin

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            var servers = eval(bridge.discover_servers());
            serverListModel.clear();
            servers.map(function (server) {
                serverListModel.append({
                    ip: server.ip.join("."),
                    port: server.port,
                    name: server.name
                });
            });
        }
    }

    ColumnLayout {
        id: mainLayout
        /* anchors.fill: parent */
        anchors.margins: margin

        Label {
            id: infoLabel
            text: "Gefundene Server:"
        }

        ListView {
            id: serverList
            width: 200
            height: 200
            /* Layout.fillHeight: true */
            model: ListModel {
                id: serverListModel
                ListElement {
                    ip: "0.0.0.0"
                    port: 0
                    name: "Warte auf Server..."
                }
            }
            delegate: Item {
                x: 5
                width: 80
                height: 15
                Row {
                    id: row1
                    spacing: 10
                    anchors.verticalCenter: parent.verticalCenter
                    Rectangle {
                        width: 200
                        height: 15

                        Text {
                            text: name + " (" + ip + ":" + port + ")"
                            anchors.verticalCenter: parent.verticalCenter
                            font.bold: true
                        }

                        MouseArea {
                            id: mouse_area1
                            z: 1
                            hoverEnabled: true
                            anchors.fill: parent

                            onClicked:{
                                serverList.currentIndex = index
                                hostnameField.text = ""
                                portField.text = ""
                                console.log("Chose server " + index);
                            }
                        }
                    }
                }
            }
        }

        TextField {
            id: nicknameField
            Layout.fillWidth: true
            placeholderText: "Nickname"
            focus: true
        }

        RowLayout {
            TextField {
                id: hostnameField
                Layout.fillWidth: true
                placeholderText: "Host"
            }

            TextField {
                id: portField
                Layout.fillWidth: true
                placeholderText: "Port"
            }
        }

        Button {
            text: "Verbinden"
            onClicked: connect()
        }
    }

    function connect() {
        hide();
        if (hostnameField.text != "") {
            bridge.connect(hostnameField.text, parseInt(portField.text), nicknameField.text);
        } else {
            bridge.connect(serverListModel.get(serverList.currentIndex).ip, serverListModel.get(serverList.currentIndex).port, nicknameField.text);
        }
        var component = Qt.createQmlObject(assets.get_main_window(), connectWindow, "main_window");
        component.show();
    }
}
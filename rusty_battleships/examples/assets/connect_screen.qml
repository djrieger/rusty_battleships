import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

Item {
    id: screen

    visible: false

    signal connected();

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
            onClicked: {
              // FIXME: validate inputs, handle errors
              connect();
            }
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

    function connect() {
        if (hostnameField.text != "") {
          bridge.connect(hostnameField.text, parseInt(portField.text, 10), nicknameField.text);
        } else {
          var server = serverListModel.get(serverList.currentIndex);
          bridge.connect(server.ip, parseInt(server.port, 10), nicknameField.text);
        }

        // TODO: wait for response, handle errors

        screen.connected();
    }


    function updateServers() {
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

    function activate() {
      timer.triggered.connect(updateServers);
      visible = true;
    }

    function deactivate() {
      timer.triggered.disconnect(updateServers);
      visible = false;
    }
}

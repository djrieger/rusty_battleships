import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Controls.Styles 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

Item {
    id: screen

    anchors.fill: parent
    visible: false

    signal connected();

    GroupBox {
        anchors.left: parent.left
        anchors.right: parent.right

        title: "Connect to game server"

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 5
            spacing: 12

            Label {
                text: "Please select a server from the list."
            }

            ColumnLayout {
                Layout.fillWidth: true

                ComboBox {
                    id: serverList

                    editable: false
                    enabled: false // initially disabled
                    Layout.fillWidth: true

                    model: ListModel {
                        id: serverListModel
                        ListElement { name: "Other server…" }
                    }
                    // actual server selected, not "other server"?
                    property bool serverSelected: currentIndex !== count - 1
                    textRole: "name"

                    property string previousValue
                    onEnabledChanged: {
                        // remember previous value during an update
                        if (enabled) {
                            if (previousValue) {
                                if (find(previousValue) !== -1) {
                                    currentIndex = find(previousValue);
                                } else {
                                    // choose "Other server…" when server has disappeared
                                    currentIndex = count - 1;
                                }
                            }
                        } else {
                            if (currentIndex !== -1) {
                                previousValue = currentText;
                            }
                        }
                    }
                }

                RowLayout {
                    id: updateNotice

                    BusyIndicator {
                        implicitHeight: 10; implicitWidth: 10
                    }
                    Label {
                        text: "Searching for available game servers"
                    }
                }

                TextField {
                    id: customServer
                    visible: !serverList.serverSelected && serverList.enabled
                    Layout.fillWidth: true

                    placeholderText: "Example: 127.0.0.1:5000"

                    property bool validIp: false

                    style: TextFieldStyle {
                        background: Rectangle {
                            radius: 2
                            border.color: customServer.validIp ? "black" : "red"
                            border.width: 1
                        }
                    }

                    onTextChanged: {
                        validIp = false;
                        var components = text.split(":");

                        if (components.length === 2) {
                            var ipOctets = components[0].split(".");
                            var port = parseInt(components[1], 10);

                            if (ipOctets.length === 4 && port >= 0 && port <= 65535) {
                                var valid = true;

                                for (var i = 0; i < 4; i++) {
                                    var octet = parseInt(ipOctets[i], 10);
                                    // this way, we also check whether octet is an actual integer
                                    valid = valid && octet >= 0 && octet <= 255;
                                }

                                validIp = valid;
                            }
                        }
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true

                RowLayout {
                    Button {
                        anchors.topMargin: 50
                        enabled: (serverList.serverSelected || customServer.validIp) && !connecting
                        text: "Connect"

                        property bool connecting: false

                        onClicked: {
                            connecting = true
                            connectNotice.visible = true
                            connectError.visible = false

                            var ip;
                            var port;

                            if (serverList.serverSelected) {
                                var server = serverListModel.get(serverList.currentIndex);
                                ip = server.ip;
                                port = server.port;
                            } else {
                                var components = customServer.text.split(":");
                                ip = components[0];
                                port = parseInt(components[1], 10);
                            }

                            if (bridge.connect(ip, port)) {
                                screen.connected();
                            } else {
                                connectError.visible = true
                            }

                            connectNotice.visible = false
                            connecting = false
                        }
                    }
                    Label {
                        id: connectError

                        color: "red"
                        text: "Error while connecting to server"
                        visible: false
                    }
                }

                RowLayout {
                    id: connectNotice

                    visible: false

                    BusyIndicator {
                        implicitHeight: 10; implicitWidth: 10
                    }
                    Label {
                        text: "Connecting to server"
                    }
                }
            }
        }
    }

    function updateServers() {
        // don't annoy the user by changing anything while he's using the box
        if (!serverList.pressed) {
            var servers = eval(bridge.discover_servers());
            serverListModel.clear();
            servers.map(function (server) {
                serverListModel.append({
                    ip: server.ip.join("."),
                    port: server.port,
                    name: server.name
                });
            });
            serverListModel.append({
                name: "Other server…"
            });

            serverList.enabled = true;
            updateNotice.visible = false;
        }
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

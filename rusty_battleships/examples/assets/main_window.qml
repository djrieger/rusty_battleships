import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0

ApplicationWindow {
  visible: true
  title: "Battleships"

  property int margin: 11
  width: mainLayout.implicitWidth + 2 * margin
  height: mainLayout.implicitHeight + 2 * margin
  minimumWidth: mainLayout.Layout.minimumWidth + 2 * margin
  minimumHeight: mainLayout.Layout.minimumHeight + 2 * margin

  ColumnLayout {
    id: mainLayout
    anchors.fill: parent
    anchors.margins: margin

    Timer {
      interval: 500; running: true; repeat: true
      onTriggered: time.text = Date().toString()
    }

    Text { id: time }
  }
}

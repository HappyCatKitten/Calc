#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QJsonDocument>
#include <QClipboard>
#include <QIcon>
extern "C" {void* calc_new();void calc_free(void*);char* calc_action(void*,const char*);void calc_string_free(char*);}
class Calculator : public QObject {
 Q_OBJECT
public:
 Calculator():core(calc_new()){} ~Calculator(){calc_free(core);}
 Q_INVOKABLE QVariantMap action(const QString& key){auto bytes=key.toUtf8();auto s=calc_action(core,bytes.constData());auto result=QJsonDocument::fromJson(s).toVariant().toMap();calc_string_free(s);return result;}
 Q_INVOKABLE QString paste(){return QGuiApplication::clipboard()->text();}
 Q_INVOKABLE void copy(const QString& value){QGuiApplication::clipboard()->setText(value);}
private:void* core;
};
int main(int argc,char** argv){QGuiApplication app(argc,argv);app.setApplicationName("Calc");app.setOrganizationName("HappyCatKitten");app.setApplicationVersion("1.2.1");app.setDesktopFileName("calc");app.setWindowIcon(QIcon(":/icon.svg"));
Calculator calculator;QQmlApplicationEngine engine;engine.rootContext()->setContextProperty("calculator",&calculator);engine.load(QUrl("qrc:/qml/Main.qml"));if(engine.rootObjects().isEmpty())return 1;return app.exec();}
#include "bridge.moc"

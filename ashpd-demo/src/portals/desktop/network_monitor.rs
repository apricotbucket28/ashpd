use adw::prelude::*;
use ashpd::{desktop::network_monitor::NetworkMonitorProxy, zbus};
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/network_monitor.ui")]
    pub struct NetworkMonitorPage {
        #[template_child]
        pub network_available: TemplateChild<gtk::Label>,
        #[template_child]
        pub metered: TemplateChild<gtk::Label>,
        #[template_child]
        pub connectivity: TemplateChild<gtk::Label>,
        #[template_child]
        pub host_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub port_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub can_reach_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NetworkMonitorPage {
        const NAME: &'static str = "NetworkMonitorPage";
        type Type = super::NetworkMonitorPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<adw::ClampLayout>();
            klass.install_action(
                "network_monitor.can_reach",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        if let Err(err) = page.can_reach().await {
                            tracing::error!("Failed to call can reach on NetworkMonitor {}", err);
                        }
                    }));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for NetworkMonitorPage {}
    impl WidgetImpl for NetworkMonitorPage {
        fn map(&self, widget: &Self::Type) {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak widget => async move {
                if let Err(err) = widget.refresh().await {
                    tracing::error!("Failed to call can refresh on NetworkMonitor {}", err);
                }
            }));

            self.parent_map(widget);
        }
    }
    impl BinImpl for NetworkMonitorPage {}
}

glib::wrapper! {
    pub struct NetworkMonitorPage(ObjectSubclass<imp::NetworkMonitorPage>) @extends gtk::Widget, adw::Bin;
}

impl NetworkMonitorPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a NetworkMonitorPage")
    }

    async fn refresh(&self) -> ashpd::Result<()> {
        let self_ = imp::NetworkMonitorPage::from_instance(self);
        let cnx = zbus::azync::Connection::session().await?;
        let proxy = NetworkMonitorProxy::new(&cnx).await?;
        let status = proxy.status().await?;

        self_
            .connectivity
            .set_label(&status.connectivity.to_string());
        self_
            .network_available
            .set_label(&status.available.to_string());
        self_.metered.set_label(&status.metered.to_string());

        Ok(())
    }

    async fn can_reach(&self) -> ashpd::Result<()> {
        let self_ = imp::NetworkMonitorPage::from_instance(self);
        let cnx = zbus::azync::Connection::session().await?;
        let proxy = NetworkMonitorProxy::new(&cnx).await?;

        let hostname = self_.host_entry.text();
        let port = self_.port_entry.text().parse().unwrap_or(80);
        let response = proxy.can_reach(&hostname, port).await?;

        self_.can_reach_row.set_title(Some(&response.to_string()));
        self_.response_group.show();
        Ok(())
    }
}

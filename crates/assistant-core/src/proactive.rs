use crate::domain::{
    AvatarMood, CanonicalEvent, PhoneDevice, PhoneSignalRecord, ProactiveAlert, UpcomingDate,
};

/// Avalia dados pessoais (aniversários, datas comemorativas, agenda e celular)
/// para produzir alertas contextuais proativos sem spam repetitivo.
pub fn build_proactive_alerts(
    now_ts: i64,
    upcoming_dates: &[UpcomingDate],
    today_events: &[CanonicalEvent],
    phones: &[PhoneDevice],
    recent_signals: &[PhoneSignalRecord],
) -> Vec<ProactiveAlert> {
    let mut alerts = Vec::new();

    // 1. Ligações ou bateria crítica no celular têm prioridade imediata
    for sig in recent_signals.iter().take(5) {
        if sig.kind == "call" && now_ts - sig.ts <= 90 {
            alerts.push(ProactiveAlert {
                key: format!("call:{}", sig.id),
                level: "urgent".into(),
                avatar_mood: AvatarMood::Alert,
                icon: "📞".into(),
                title: format!("Ligação no celular: {}", sig.title),
                body: if sig.body.is_empty() {
                    "Chamada recebida no aparelho conectado".into()
                } else {
                    sig.body.clone()
                },
                bubble_text: format!("📞 {}", sig.title),
                action_label: "Ver Celular".into(),
                action_target: "tab:assistant:celular".into(),
                created_ts: now_ts,
            });
            break;
        }
    }

    for p in phones {
        if p.connected && !p.charging {
            if let Some(pct) = p.battery_pct {
                if pct <= 15 {
                    alerts.push(ProactiveAlert {
                        key: format!("phone-bat:{}:{}", p.id, pct / 5),
                        level: "urgent".into(),
                        avatar_mood: AvatarMood::Alert,
                        icon: "🪫".into(),
                        title: format!("{} com bateria baixa ({}%)", p.name, pct),
                        body: "Conecte seu celular ao carregador".into(),
                        bubble_text: format!("🪫 {}% no celular", pct),
                        action_label: "Celular".into(),
                        action_target: "tab:assistant:celular".into(),
                        created_ts: now_ts,
                    });
                }
            }
        }
    }

    // 2. Aniversários e datas comemorativas hoje ou amanhã ativam modo celebração no avatar
    for d in upcoming_dates {
        if d.days_until == 0 {
            let icon = if d.kind == "birthday" { "🎂" } else { "🎉" };
            let age_txt = d
                .turning_age
                .map(|a| format!(" ({a} anos)"))
                .unwrap_or_default();
            alerts.push(ProactiveAlert {
                key: format!("date:{}:{}:0", d.id, d.day),
                level: "celebrate".into(),
                avatar_mood: AvatarMood::Celebrate,
                icon: icon.into(),
                title: format!("Hoje: {}{}", d.title, age_txt),
                body: "Aproveite para enviar uma mensagem ou felicitação!".into(),
                bubble_text: format!("{icon} {}", d.title),
                action_label: "Ver Datas".into(),
                action_target: "tab:assistant:hoje".into(),
                created_ts: now_ts,
            });
        } else if d.days_until == 1 {
            let icon = if d.kind == "birthday" { "🎁" } else { "🗓️" };
            alerts.push(ProactiveAlert {
                key: format!("date:{}:{}:1", d.id, d.day),
                level: "normal".into(),
                avatar_mood: AvatarMood::Celebrate,
                icon: icon.into(),
                title: format!("Amanhã: {}", d.title),
                body: "Lembrete antecipado da sua agenda pessoal".into(),
                bubble_text: format!("{icon} Amanhã: {}", d.title),
                action_label: "Agenda".into(),
                action_target: "tab:assistant:hoje".into(),
                created_ts: now_ts,
            });
        }
    }

    // 3. Reunião nos próximos 10 minutos
    for ev in today_events {
        if ev.all_day {
            continue;
        }
        let diff = ev.start_ts - now_ts;
        if diff > 0 && diff <= 600 {
            let mins = ((diff + 59) / 60).max(1);
            alerts.push(ProactiveAlert {
                key: format!("meet:{}", ev.fingerprint),
                level: "urgent".into(),
                avatar_mood: AvatarMood::Meeting,
                icon: "📹".into(),
                title: format!("Em {mins} min: {}", ev.title),
                body: format!("{} · {}", ev.start_time, ev.source_provider.as_str()),
                bubble_text: format!("📹 {mins}m: {}", ev.title),
                action_label: if ev.join_url.is_empty() { "Agenda".into() } else { "Entrar".into() },
                action_target: if ev.join_url.is_empty() {
                    "tab:assistant:hoje".into()
                } else {
                    ev.join_url.clone()
                },
                created_ts: now_ts,
            });
            break;
        }
    }

    alerts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ProviderKind;

    #[test]
    fn generates_celebration_alert_for_birthday_today() {
        let dates = vec![UpcomingDate {
            id: "b1".into(),
            title: "Aniversário de Ana".into(),
            person_name: "Ana".into(),
            kind: "birthday".into(),
            month: 11,
            day: 13,
            days_until: 0,
            turning_age: Some(28),
            source_provider: ProviderKind::Google,
        }];
        let alerts = build_proactive_alerts(1_700_000_000, &dates, &[], &[], &[]);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].avatar_mood, AvatarMood::Celebrate);
        assert!(alerts[0].title.contains("28 anos"));
    }
}

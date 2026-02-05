use {
    doubloon::{
        Currency,
        Money,
    },
    async_proto_derive::impl_protocol_for,
    crate::Protocol,
};

#[derive(Protocol)]
#[async_proto(internal)]
struct MoneyProxy<C> {
    amount: rust_decimal::Decimal,
    currency: C,
}

impl<C: Copy> From<MoneyProxy<C>> for Money<C> {
    fn from(value: MoneyProxy<C>) -> Self {
        Self::new(value.amount, value.currency)
    }
}

impl<'a, C: Currency + Copy> From<&'a Money<C>> for MoneyProxy<C> {
    fn from(value: &'a Money<C>) -> Self {
        Self {
            amount: value.amount(),
            currency: value.currency(),
        }
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    #[async_proto(via = MoneyProxy<C>, where(C: Protocol + Currency + Copy + Send + Sync + 'static))]
    type Money<C>;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::AED;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::AFN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::ALL;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::AMD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::AOA;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::ARS;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::AUD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::AWG;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::AZN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BAM;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BBD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BDT;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BHD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BIF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BMD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BND;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BOB;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BOV;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BRL;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BSD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BTN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BWP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BYN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::BZD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CAD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CDF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CHE;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CHF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CHW;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CLF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CLP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CNY;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::COP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::COU;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CRC;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CUP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CVE;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::CZK;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::DJF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::DKK;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::DOP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::DZD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::EGP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::ERN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::ETB;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::EUR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::FJD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::FKP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::GBP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::GEL;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::GHS;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::GIP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::GMD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::GNF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::GTQ;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::GYD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::HKD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::HNL;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::HTG;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::HUF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::IDR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::ILS;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::INR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::IQD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::IRR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::ISK;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::JMD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::JOD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::JPY;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::KES;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::KGS;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::KHR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::KMF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::KPW;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::KRW;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::KWD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::KYD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::KZT;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::LAK;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::LBP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::LKR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::LRD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::LSL;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::LYD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MAD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MDL;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MGA;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MKD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MMK;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MNT;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MOP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MRU;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MUR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MVR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MWK;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MXN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MXV;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MYR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::MZN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::NAD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::NGN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::NIO;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::NOK;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::NPR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::NZD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::OMR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::PAB;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::PEN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::PGK;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::PHP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::PKR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::PLN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::PYG;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::QAR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::RON;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::RSD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::RUB;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::RWF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SAR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SBD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SCR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SDG;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SEK;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SGD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SHP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SLE;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SOS;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SRD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SSP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::STN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SVC;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SYP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::SZL;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::THB;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::TJS;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::TMT;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::TND;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::TOP;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::TRY;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::TTD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::TWD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::TZS;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::UAH;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::UGX;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::USD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::USN;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::UYI;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::UYU;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::UYW;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::UZS;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::VED;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::VES;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::VND;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::VUV;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::WST;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XAD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XAF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XAG;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XAU;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XBA;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XBB;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XBC;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XBD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XCD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XCG;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XDR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XOF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XPD;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XPF;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XPT;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XSU;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XTS;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XUA;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::XXX;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::YER;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::ZAR;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::ZMW;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "doubloon")))))]
    struct doubloon::iso_currencies::ZWG;
}

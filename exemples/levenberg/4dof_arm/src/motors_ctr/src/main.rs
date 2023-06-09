use futures::{StreamExt, future};
use r2r;
use r2r::messages::msg::MotorsAngles;
use r2r::qos::QosProfile;

use linux_embedded_hal::I2cdev;
use sna41_motorshield::{servo::ServoNumber, MotorShield};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create ros node and subscriber to get theta_1 and theta_2 command
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "motors_ctr", "")?;
    println!("node name: {}", node.name()?);
    let subscriber = node.subscribe::<MotorsAngles>("/motors_angles", QosProfile::default())?;

    // Create I2C to communicate with motors
    let i2c = I2cdev::new("/dev/i2c-1").expect("Error creating I2c");
    let mut ms = MotorShield::new(i2c).expect("Error creating motorshield interface");

    // Put all servo motors to default position 
    ms.set_servo_angle(ServoNumber::S0, 0.0).expect("Error setting motor");
    ms.set_servo_angle(ServoNumber::S1, 90.0).expect("Error setting motor");
    ms.set_servo_angle(ServoNumber::S2, 0.0).expect("Error setting motor");
    ms.set_servo_angle(ServoNumber::S3, 0.0).expect("Error setting motor");

    // Run in a tokio task
    tokio::task::spawn(async move {

        subscriber.for_each(|msg| {
            println!("New command: theta_1 = {}°, theta_2 = {}°, theta_3 = {}°, theta_4 = {}°", msg.theta_1, msg.theta_2, msg.theta_3, msg.theta_4);
            ms.set_servo_angle(ServoNumber::S0, msg.theta_1 as f32).expect("Error setting motor");
            ms.set_servo_angle(ServoNumber::S1, msg.theta_2 as f32).expect("Error setting motor");
            ms.set_servo_angle(ServoNumber::S2, 90_f32 + msg.theta_3 as f32).expect("Error setting motor");
            ms.set_servo_angle(ServoNumber::S3, 90_f32 + msg.theta_4 as f32).expect("Error setting motor");
            
            future::ready(())
        })
        .await
    });


    loop {
        node.spin_once(std::time::Duration::from_millis(100));
    }
}
